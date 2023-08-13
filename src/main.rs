mod syscall;
use libc;

use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::os::fd;

// Read these document before develpment.
// * Nginx Development Guide
// http://nginx.org/en/docs/dev/development_guide.html#code_layout

// * Deal Unsafe Rust
// https://doc.rust-jp.rs/rust-nomicon-ja/meet-safe-and-unsafe.html

const MAX_EVENTS_SIZE: i32 = 1024;
const TIMEOUT_CLOCKS: i32 = 100;

#[derive(Clone)]
struct Event {
    pub fd: fd::RawFd,
    pub wait_for_read: bool,
    pub wait_for_write: bool
}

impl Event {
    pub fn init_read_event(fd: fd::RawFd) -> Event{
        Event {
            fd,
            wait_for_read: true,
            wait_for_write: false
        }
    }

    pub fn init_write_event(fd: fd::RawFd) -> Event{
        Event {
            fd,
            wait_for_read: false,
            wait_for_write: true
        }
    }
}

fn main() {
    let addr = libc::sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_port: (8080 as u16).to_be(), // htons(8080)
        sin_addr: libc::in_addr {
            s_addr: libc::INADDR_ANY,
        },
        sin_zero: [0; 8],
    };

    println!("Start Server!");
    let listener_fd = syscall::socket().unwrap();
    let mut addr = unsafe { mem::transmute::<libc::sockaddr_in, libc::sockaddr>(addr) };

    //Bind and Listen
    // https://linuxjm.osdn.jp/html/LDP_man-pages/man2/listen.2.html
    syscall::bind(listener_fd, &addr).unwrap();
    syscall::listen(listener_fd, 10).unwrap();

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
        events: libc::EPOLLIN as u32,
        u64: listener_fd as u64,
    };
    syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, listener_fd, Some(&mut event)).unwrap();
    let mut events_buffer = unsafe { vec![mem::zeroed::<libc::epoll_event>(); MAX_EVENTS_SIZE as usize] };
    let mut event_map: HashMap<fd::RawFd, Event> = HashMap::new();

    while !term.load(Ordering::Relaxed) {
        // epollにeventが入ってくるまで待機
        let wait_result =
            syscall::epoll_wait(epoll_fd, &mut events_buffer, MAX_EVENTS_SIZE, TIMEOUT_CLOCKS);
        let events_num = match wait_result {
            Ok(n) => n,
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
                let accept_fd = syscall::accept(listener_fd, &mut addr).unwrap();
                if accept_fd == -1 {
                    println!("Error");
                    continue;
                }
                println!("Accept connection. Prepare a file descriptor {} for this connection.", &accept_fd);
                let mut epoll_event = libc::epoll_event {
                    events: libc::EPOLLIN as u32,
                    u64: accept_fd as u64,
                };
                syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, accept_fd, Some(&mut epoll_event)).unwrap();
                let event = Event::init_read_event(accept_fd);
                event_map.insert(accept_fd, event);
                continue;
            }

            // Pop event from the event_map
            let flags = events_buffer[n].events as i32;
            println!("Flags: {}", flags as i32);
            let event_option = event_map.get(&event_fd).cloned();

            if let Some(event) = event_option {
                // Process Read Event
                let is_readable = (flags & libc::EPOLLIN) > 0;
                println!("Read Flag{}", flags & libc::EPOLLIN);
                if is_readable && event.wait_for_read {
                    let mut buf = [0 as u8; 1024];
                    println!("Get ready to read from {}.", &event_fd);
                    syscall::read(event_fd, &mut buf).unwrap();
                    let s = String::from_utf8_lossy(&buf);
                    println!("Recv: {}", s);

                    // Wait until the write event is ready.
                    let write_event = Event::init_write_event(event_fd);
                    event_map.insert(event_fd, write_event);
                    let mut epoll_event = libc::epoll_event {
                        events: libc::EPOLLOUT as u32,
                        u64: event_fd as u64,
                    };
                    syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_MOD, event_fd, Some(&mut epoll_event)).unwrap();
                }

                // Process Write Event
                let is_writable = (flags & libc::EPOLLOUT) > 0;
                if is_writable & event.wait_for_write {
                    let send_str = String::from("HTTP/1.1 204 No Content\r\n\r\n");
                    let mut send_buf = send_str.clone().into_bytes();
                    println!("Send: {}", &send_str);
                    syscall::write(event_fd, &mut send_buf).unwrap();
                    syscall::shutdown(event_fd).unwrap();
                    syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_DEL, event_fd, None).unwrap();
                    continue;
                }
            } else {
                // Something wrong
                println!("Something wrong");
                syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_DEL, event_fd, None).unwrap();
            }
        }
    }

    // Close
    println!("End Server!");
    unsafe {
        libc::close(epoll_fd);
        libc::close(listener_fd);
    }
}
