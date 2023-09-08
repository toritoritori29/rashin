use bytes::Bytes;

pub struct HTTPHeader {
    pub method_start: usize,
    pub method_end: usize,
    pub path_start: usize,
    pub path_end: usize,
    pub protocol_start: usize,
    pub protocol_end: usize,
}

impl HTTPHeader {
    pub fn new() -> Self {
        HTTPHeader {
            method_start: 0,
            method_end: 0,
            path_start: 0,
            path_end: 0,
            protocol_start: 0,
            protocol_end: 0,
        }
    }

    pub fn method<'a>(&self, buffer: &'a Bytes) -> &'a str {
        std::str::from_utf8(&buffer[self.method_start..self.method_end]).unwrap()
    }

    pub fn path<'a>(&self, buffer: &'a Bytes) -> &'a str {
        std::str::from_utf8(&buffer[self.path_start..self.path_end]).unwrap()
    }

    pub fn protocol<'a>(&self, buffer: &'a Bytes) -> &'a str {
        std::str::from_utf8(&buffer[self.protocol_start..self.protocol_end]).unwrap()
    }
}
pub enum ParseResult {
    Again,
    Ok,
    Complete,
    Error,
}
