mod syscall;
use libc;

use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

    println!("Start Server!");
    // let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
    let fd = syscall::socket().unwrap();
    let mut addr = unsafe { mem::transmute::<libc::sockaddr_in, libc::sockaddr>(addr) };

    //Bind and Listen
    // https://linuxjm.osdn.jp/html/LDP_man-pages/man2/listen.2.html
    syscall::bind(fd, &addr).unwrap();
    syscall::listen(fd, 10).unwrap();

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
        u64: fd as u64,
    };
    syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, fd, Some(&mut event)).unwrap();
    let mut events = unsafe { vec![mem::zeroed::<libc::epoll_event>(); MAX_EVENTS_SIZE as usize] };
    while !term.load(Ordering::Relaxed) {
        let events_num =
            syscall::epoll_wait(epoll_fd, &mut events, MAX_EVENTS_SIZE, TIMEOUT_CLOCKS).unwrap();

        for n in 0..events_num {
            let active_fd = events[n as usize].u64;
            println!("Event: {}", active_fd);
            if active_fd as i32 == fd {
                let accept_fd = syscall::accept(fd, &mut addr).unwrap();
                if accept_fd == -1 {
                    println!("Error");
                    continue;
                }
                println!("Accept connection. Prepare a file descriptor {} for this connection.", $accept_fd);
                // Register to epoll
                let mut connection_event = libc::epoll_event {
                    events: libc::EPOLLIN as u32,
                    u64: epoll_fd as u64,
                };
                syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, accept_fd, Some(&mut connection_event)).unwrap();
            } else {
                let accept_fd = active_fd as i32;
                let mut buf = [0 as u8; 1024];

                // Read
                println!("Get ready to read from {}.", &accept_fd);
                syscall::read(accept_fd, &mut buf).unwrap();
                println!("Read");
                let s = String::from_utf8_lossy(&buf);
                println!("Recv: {}", s);

                let send_str = String::from("HTTP/1.1 204 No Content\r\n\r\n");
                let mut send_buf = send_str.clone().into_bytes();
                println!("Send: {}", &send_str);
                syscall::write(accept_fd, &mut send_buf).unwrap();
                syscall::shutdown(accept_fd).unwrap();
                syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_DEL, accept_fd, None).unwrap();
            }
        }
    }

    // Close
    println!("End Server!");
    unsafe {
        libc::close(fd);
    }
}
