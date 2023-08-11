use std::mem;
use std::os::fd::{self, AsRawFd};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RashinErr {
    #[error("Syscall returns some error.")]
    SyscallError,
}

pub fn socket() -> Result<std::os::fd::RawFd, RashinErr> {
    let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
    if fd == -1 {
        return Err(RashinErr::SyscallError);
    }
    Ok(fd)
}

pub fn bind(fd: fd::RawFd, addr: &libc::sockaddr) -> Result<(), RashinErr> {
    let addr_size = mem::size_of::<libc::sockaddr>() as u32;
    let error_code = unsafe { libc::bind(fd.as_raw_fd(), addr, addr_size) };
    if error_code == -1 {
        return Err(RashinErr::SyscallError);
    }
    Ok(())
}

pub fn listen(fd: fd::RawFd, backlog: i32) -> Result<(), RashinErr> {
    let error_code = unsafe { libc::listen(fd.as_raw_fd(), backlog) };
    if error_code == -1 {
        return Err(RashinErr::SyscallError);
    }
    Ok(())
}

/// SocketからデータをBufferに読み込む
/// データを読み込むためにはread, recv, recvfrom, recvmsgなどのシステムコールを使用することができる
/// メッセージがBufferのサイズを超える場合は、メッセージが切り捨てられる
///
/// 参考1. Rust本体のTcpListener周りの実装
/// https://github.com/rust-lang/rust/blob/11467b1c2a56bd2fd8272a7413190c814cfcba1f/library/std/src/sys/unix/net.rs#L260
pub fn read(fd: i32, buf: &mut [u8]) -> Result<(), RashinErr> {
    let size = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
    println!("buf: {:?}", size);
    if size < 0 {
        return Err(RashinErr::SyscallError);
    }
    Ok(())
}

/// Socketにデータを書き込む
/// データを書き込むためにはwrite, send, sendto, sendmsgなどのシステムコールを使用することができる
/// 現時点ではwriteで十分なのでwriteを使用する
///
/// 参考1. Manpage
/// https://linuxjm.osdn.jp/html/LDP_man-pages/man2/send.2.html
pub fn write(fd: i32, buf: &mut [u8]) -> Result<&[u8], RashinErr> {
    let size =
        unsafe { libc::write(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
    if size == -1 {
        return Err(RashinErr::SyscallError);
    }
    Ok(&buf[..size as usize])
}

pub fn accept(fd: i32, addr: &mut libc::sockaddr) -> Result<i32, RashinErr> {
    let mut addr_size = mem::size_of::<libc::sockaddr>() as u32;
    let accept_fd = unsafe { libc::accept(fd, addr, &mut addr_size) };
    if accept_fd == -1 {
        return Err(RashinErr::SyscallError);
    }
    Ok(accept_fd)
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
pub fn shutdown(fd: i32) -> Result<(), RashinErr> {
    let error_code = unsafe { libc::shutdown(fd, libc::SHUT_WR) };
    if error_code == -1 {
        return Err(RashinErr::SyscallError);
    }
    Ok(())
}

pub fn epoll_create() -> Result<fd::RawFd, RashinErr>{
    // Initialize epoll
    // fdがread可能かどうか監視する
    // epoll_createとepoll_create1が存在しているが大きな違いは無い
    let epoll_fd = unsafe { libc::epoll_create1(0) };
    if epoll_fd == -1 {
        return Err(RashinErr::SyscallError);
    }
    Ok(epoll_fd)
}
