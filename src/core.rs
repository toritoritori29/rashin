use bytes::Bytes;
/// Core.rs
/// このモジュールではrashinの基本的な構造体の定義と, イベントハンドラの定義を行う
use std::os::fd::RawFd;

use crate::error::RashinErr;
use crate::http::http_interface::{HTTPHeader, ParseResult};
use crate::http::parse_request_header::parse_http_request_line;
use crate::syscall;

#[derive(Clone, Debug)]
pub enum EventState {
    Ready,
    Shutdown,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub readable: bool,
    pub writable: bool,
    pub state: EventState,
    pub handler: fn(RawFd, &mut Event),
    pub connection: Option<Connection>,
}

impl Event {
    pub fn is_ready(&self) -> bool {
        match self.state {
            EventState::Ready => true,
            EventState::Shutdown => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Connection {
    pub fd: RawFd,
    pub buf: Vec<u8>,
}

impl Connection {
    pub fn new(fd: RawFd) -> Connection {
        Connection {
            fd: fd,
            buf: vec![0 as u8; 1024],
        }
    }
}

pub fn init_http_event(connection: Connection) -> Event {
    Event {
        readable: false,
        writable: false,
        state: EventState::Ready,
        handler: http_handler,
        connection: Some(connection),
    }
}

pub fn http_handler(fd: RawFd, event: &mut Event) {
    if !event.is_ready() {
        println!("Not ready");
        return;
    }

    if !(event.readable && event.writable) {
        return;
    }
    println!("Get ready to read from {}.", &fd);
    if let Some(connection) = &mut event.connection {
        let read_option = syscall::read(fd, &mut connection.buf);
        let size = match read_option {
            Ok(size) => size as usize,
            Err(RashinErr::SyscallError(libc::EAGAIN)) => {
                event.readable = false;
                return;
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        };

        let mut header = HTTPHeader::new();
        let buffer = Bytes::copy_from_slice(&connection.buf);

        let result = parse_http_request_line(&buffer, &mut header);
        if let ParseResult::Complete = result {
            println!("Method: {}", header.method(&buffer));
            println!("Path: {}", header.path(&buffer));
            println!("Protocol: {}", header.protocol(&buffer));
        } else {
            println!("Parse Error");
        }

        // Process Write Event
        let send_str = String::from("HTTP/1.1 204 No Content\r\n\r\n");
        let mut send_buf = send_str.clone().into_bytes();
        println!("Send: {}", &send_str);
        syscall::write(fd, &mut send_buf).unwrap();

        // Register Write Event
        // event_map.insert(event_fd, write_event);
        // let mut epoll_event = libc::epoll_event {
        //     events: libc::EPOLLOUT as u32,
        //     u64: event_fd as u64,
        // };
        // syscall::epoll_ctl(epoll_fd, libc::EPOLL_CTL_MOD, event_fd, Some(&mut epoll_event)).unwrap();
        event.state = EventState::Shutdown;
    } else {
        println!("Connection is None.");
    }
}
