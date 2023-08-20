use std::os::fd::RawFd;
use crate::syscall;


#[derive(Clone)]
pub enum EventState {
    Ready,
    Shutdown,
}

#[derive(Clone)]
pub struct Event {
    pub readable: bool,
    pub writable: bool,
    pub state: EventState,
    pub handler: fn(RawFd, &mut Event),
}

impl Event {
    pub fn is_ready(&self) -> bool {
        match self.state {
            EventState::Ready => true,
            EventState::Shutdown => false,
        }
    }
}

pub fn init_http_event() -> Event{
    Event {
        readable: false,
        writable: false,
        state: EventState::Ready,
        handler: http_handler
    }
}

pub fn http_handler(fd: RawFd, event: &mut Event) {
    if !event.is_ready() {
        return;
    }
    if event.readable && event.writable {
        return;
    }
    let mut buf = [0 as u8; 1024];
    println!("Get ready to read from {}.", &fd);
    syscall::read(fd, &mut buf).unwrap();
    let s = String::from_utf8_lossy(&buf);
    println!("Recv: {}", s);

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
}

