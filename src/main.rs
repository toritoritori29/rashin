extern crate libc;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// Read these document before develpment.
// https://doc.rust-jp.rs/rust-nomicon-ja/meet-safe-and-unsafe.html

fn main() {
    let addr = libc::sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_port: (8080 as u16).to_be(),
        sin_addr: libc::in_addr {
            s_addr: libc::INADDR_ANY,
        },
        sin_zero: [0; 8],
    };

    println!("Start Server!");
    unsafe{
        let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0);
        let mut addr = mem::transmute::<libc::sockaddr_in, libc::sockaddr>(addr);
        let mut addr_size = mem::size_of::<libc::sockaddr>() as u32;

        //Bind and Listen
        // https://linuxjm.osdn.jp/html/LDP_man-pages/man2/listen.2.html
        libc::bind(fd, &addr, addr_size);
        libc::listen(fd, 10);

        // Signal Handling
        // https://docs.rs/signal-hook/latest/signal_hook/
        let term = Arc::new(AtomicBool::new(false));
        match signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)) {
            Ok(_) => {},
            Err(e) => panic!("Error: {}", e),
        };

        while !term.load(Ordering::Relaxed) {
            // Accept
            let accept_fd = libc::accept(fd,  &mut addr, &mut addr_size);
            match accept_fd {
                -1 => {
                    println!("Error");
                    continue;
                },
                _ => {
                    // Read
                    println!("Accept");
                    let mut buf: [u8; 1024] = [0; 1024];
                    libc::read(accept_fd, buf.as_mut_ptr() as *mut libc::c_void, 1024);
                    let s = match String::from_utf8(buf.to_vec()) {
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                    println!("Recv: {}", s);

                    let send_str = format!("You send: {}", s);
                    let mut send_buf = send_str.clone().into_bytes();
                    println!("Send: {}", &send_str);

                    libc::write(accept_fd, send_buf.as_mut_ptr() as *mut libc::c_void, send_buf.len());
                }
            }
        }
        println!("End Server!");
        // Close
        libc::close(fd);
    }
}