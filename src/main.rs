mod core;
mod syscall;
mod error;
mod system_utils;

use libc;

use std::collections::HashMap;
use std::mem;
use std::os::fd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::core::{init_http_event, Connection, Event, EventState};
use crate::error::RashinErr;

// Read these document before develpment.
// * Nginx Development Guide
// http://nginx.org/en/docs/dev/development_guide.html#code_layout

// * Deal Unsafe Rust
// https://doc.rust-jp.rs/rust-nomicon-ja/meet-safe-and-unsafe.html

const MAX_EVENTS_SIZE: i32 = 1024;
const TIMEOUT_CLOCKS: i32 = 100;

fn main() {
    let addr = libc::sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_port: (8080 as u16).to_be(), // htons(8080)
        sin_addr: libc::in_addr {
            s_addr: libc::INADDR_ANY,
        },
        sin_zero: [0; 8],
    };
    let mut addr = unsafe { mem::transmute::<libc::sockaddr_in, libc::sockaddr>(addr) };

    println!("Start Server!");
    let listener_fd = system_utils::create_listner_socket(&addr).unwrap();

    // Signal Handling
    // アトミック変数を用いてSIGINTが発生したか(Ctrl-Cが押されたか)を判定する
    // 参考: https://docs.rs/signal-hook/latest/signal_hook/
    let term = Arc::new(AtomicBool::new(false));
    match signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)) {
        Ok(_) => {}
        Err(e) => panic!("Error: {}", e),
    };

    let epoll_fd = syscall::epoll_create().unwrap();

    // epoll_ctlでfdを監視対象に加える
    // epoll_waitでイベントを検知した際に, ここで渡したものと同じ値を受け取ることができる
    let mut event = libc::epoll_event {
        events: (libc::EPOLLET | libc::EPOLLIN) as u32,
        u64: listener_fd as u64,
    };
    syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, listener_fd, Some(&mut event)).unwrap();
    let mut events_buffer =
        unsafe { vec![mem::zeroed::<libc::epoll_event>(); MAX_EVENTS_SIZE as usize] };
    let mut event_map: HashMap<fd::RawFd, Event> = HashMap::new();

    while !term.load(Ordering::Relaxed) {
        // epollにeventが入ってくるまで待機
        println!("Wait for epoll event");
        let wait_result = syscall::epoll_wait(
            epoll_fd,
            &mut events_buffer,
            MAX_EVENTS_SIZE,
            TIMEOUT_CLOCKS,
        );
        let events_num = match wait_result {
            Ok(n) => n,
            Err(RashinErr::SyscallError(libc::EINTR)) => {
                println!("Interrupted system call");
                continue;
            }
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        for n in 0..events_num {
            let event_fd = events_buffer[n].u64 as fd::RawFd;
            println!("Event: {}", event_fd);

            // Accept incoming connection requests.
            if event_fd == listener_fd {
                // Accept connection
                // epollで待機しているので原則としてブロックされることが無いが,
                // 何かしらの理由で当該のコネクションが消える可能性がある.
                // 詳細は`man accept`に記載があるので参照
                loop {
                    // EAGAIN または ERRORが発生するまでacceptを繰り返す
                    let accept_fd = match syscall::accept(listener_fd, &mut addr) {
                        Ok(fd) => fd,
                        Err(RashinErr::SyscallError(libc::EAGAIN)) => {
                            println!("No connection request");
                            break;
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                            break;
                        }
                    };
                    println!(
                        "Accept connection. Prepare a file descriptor {} for this connection.",
                        &accept_fd
                    );
                    let mut epoll_event = libc::epoll_event {
                        events: (libc::EPOLLET | libc::EPOLLIN | libc::EPOLLOUT) as u32,
                        u64: accept_fd as u64,
                    };
                    syscall::epoll_ctl(
                        epoll_fd,
                        libc::EPOLL_CTL_ADD,
                        accept_fd,
                        Some(&mut epoll_event),
                    )
                    .unwrap();

                    syscall::fnctl(accept_fd).unwrap();
                    let connection = Connection::new(accept_fd);
                    let event = init_http_event(connection);
                    event_map.insert(accept_fd, event);
                }
                continue;
            }

            // Pop event from the event_map
            let flags = events_buffer[n].events as i32;
            println!("Flags: {}", flags as i32);

            // 今のままだとBufferなどもコピーされるのであんまりよくない
            let event_option = event_map.get(&event_fd).cloned();

            if let Some(mut event) = event_option {
                let is_readable = (flags & libc::EPOLLIN) > 0;

                if is_readable & event.is_ready() {
                    event.readable = true;
                }

                // Process Write Event
                let is_writable = (flags & libc::EPOLLOUT) > 0;
                if is_writable & event.is_ready() {
                    event.writable = true;
                }
                (event.handler)(event_fd, &mut event);
                if let EventState::Shutdown = event.state {
                    println!("Shutdown");
                    syscall::shutdown(event_fd).unwrap();
                    syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_DEL, event_fd, None).unwrap();
                    syscall::close(event_fd).unwrap();
                }
            } else {
                // Something wrong
                println!("Something wrong");
                syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_DEL, event_fd, None).unwrap();
            }
        }
    }

    // Close
    println!("Clean up resources");
    syscall::close(epoll_fd).unwrap();
    syscall::close(listener_fd).unwrap();
    println!("End Server!");
}
