extern crate libc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{io, mem};

// Read these document before develpment.
// * Nginx Development Guide
// http://nginx.org/en/docs/dev/development_guide.html#code_layout

// * Deal Unsafe Rust
// https://doc.rust-jp.rs/rust-nomicon-ja/meet-safe-and-unsafe.html


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
    let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
    let mut addr = unsafe { mem::transmute::<libc::sockaddr_in, libc::sockaddr>(addr) };
    let mut addr_size = mem::size_of::<libc::sockaddr>() as u32;

    //Bind and Listen
    // https://linuxjm.osdn.jp/html/LDP_man-pages/man2/listen.2.html
    unsafe {
        libc::bind(fd, &addr, addr_size);
        libc::listen(fd, 10);
    }

    // Signal Handling
    // アトミック変数を用いてSIGINTが発生したか(Ctrl-Cが押されたか)を判定する
    // 参考: https://docs.rs/signal-hook/latest/signal_hook/
    let term = Arc::new(AtomicBool::new(false));
    match signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)) {
        Ok(_) => {}
        Err(e) => panic!("Error: {}", e),
    };

    while !term.load(Ordering::Relaxed) {
        let accept_fd = unsafe {
            libc::accept(fd, &mut addr, &mut addr_size)
        };

        if accept_fd == -1 {
            println!("Error");
            continue;
        }
        // Read
        println!("Accept");
        let mut buf = [0 as u8; 1024];
        read(accept_fd, &mut buf).unwrap();
        println!("Read");
        let s = String::from_utf8_lossy(&buf);
        println!("Recv: {}", s);

        let send_str = String::from("HTTP/1.1 204 No Content\r\n\r\n");
        let mut send_buf = send_str.clone().into_bytes();
        println!("Send: {}", &send_str);
        write(accept_fd, &mut send_buf).unwrap();
        shutdown(accept_fd);
    }
    // Close

    println!("End Server!");
    unsafe {
        libc::close(fd);
    }
}

/// SocketからデータをBufferに読み込む
/// データを読み込むためにはread, recv, recvfrom, recvmsgなどのシステムコールを使用することができる
/// メッセージがBufferのサイズを超える場合は、メッセージが切り捨てられる
///
/// 参考1. Rust本体のTcpListener周りの実装
/// https://github.com/rust-lang/rust/blob/11467b1c2a56bd2fd8272a7413190c814cfcba1f/library/std/src/sys/unix/net.rs#L260
fn read(fd: i32, buf: &mut [u8]) -> io::Result<()> {
    let size = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
    println!("buf: {:?}", size);
    if size > 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

/// Socketにデータを書き込む
/// データを書き込むためにはwrite, send, sendto, sendmsgなどのシステムコールを使用することができる
/// 現時点ではwriteで十分なのでwriteを使用する
/// 
/// 参考1. Manpage
/// https://linuxjm.osdn.jp/html/LDP_man-pages/man2/send.2.html
fn write(fd: i32, buf: &mut [u8]) -> io::Result<&[u8]> {
    let size =
        unsafe { libc::write(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) as usize };
    Ok(&buf[..size])
}

/// Socketを閉じる
/// 
/// manpageによるとlibc::shutdownの引数は以下の3種類を利用することができる.
/// * SHUT_RD: 読み込みを禁止する
/// * SHUT_WR: 書き込みを禁止する
/// 参考1. Rust本体のTcpListener周りの関連実装
/// https://github.com/rust-lang/rust/blob/11467b1c2a56bd2fd8272a7413190c814cfcba1f/library/std/src/sys/unix/net.rs#L379
/// 
/// 参考2. Manpage
/// https://linuxjm.osdn.jp/html/LDP_man-pages/man2/shutdown.2.html
fn shutdown(fd: i32) {
    unsafe { libc::shutdown(fd, libc::SHUT_WR) };
}
