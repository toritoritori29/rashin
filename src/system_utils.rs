/// 複数のシステムコールを組み合わせた, システム操作に関するユーティリティ

use libc;
use std::mem;
use std::os::fd;
use crate::{syscall, error::RashinErr};


/// Listenerソケットを作成し, 引数で与えられたアドレスにバインドする.
/// 作成されたソケットはノンブロッキングモードで動作する.
pub fn create_listner_socket(addr: &libc::sockaddr) -> Result<fd::RawFd, RashinErr>{
    let listener_fd = syscall::socket()?;

    // Option: SO_REUSEADDRを指定する
    // REUSEADDRを指定することで, ソケットを閉じた後にもTIME_WAIT状態にならずにすむ
    // TIME_WAIT状態だと一定時間ポートの再利用ができず, サーバーの再起動ができない
    let optval = 1;
    if let Err(e) = syscall::setsockopt(
        listener_fd,
        libc::SOL_SOCKET,
        libc::SO_REUSEADDR,
        &optval as *const _ as *const libc::c_void,
        mem::size_of::<i32>() as u32,
    ) {
        syscall::close(listener_fd).unwrap();
        return Err(e);
    }

    // Option: SO_REUSEPORTを指定する
    let optval = 1;
    if let Err(e) = syscall::setsockopt(
        listener_fd,
        libc::SOL_SOCKET,
        libc::SO_REUSEPORT,
        &optval as *const _ as *const libc::c_void,
        mem::size_of::<i32>() as u32,
   ) {
        syscall::close(listener_fd).unwrap();
        return Err(e);
    }

    // Socketを指定したアドレスにBindしてListenする
    // https://linuxjm.osdn.jp/html/LDP_man-pages/man2/listen.2.html
    if let Err(e) = syscall::bind(listener_fd, addr) {
        syscall::close(listener_fd).unwrap();
        return Err(e)
    }
    if let Err(e) = syscall::listen(listener_fd, 10) {
        syscall::close(listener_fd).unwrap();
        return Err(e)
    }
    if let Err(e) = syscall::fnctl(listener_fd) {
        syscall::close(listener_fd).unwrap();
        return Err(e)
    }
    Ok(listener_fd)
}