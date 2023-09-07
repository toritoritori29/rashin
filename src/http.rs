
use bytes::Bytes;
use std::io::{Cursor, Read};

struct HTTPHeader {
    buffer: Bytes,
    method_start: usize,
    method_end: usize,
    path_start: usize,
    path_end: usize,
    protocol_start: usize,
    protocol_end: usize,
}

impl HTTPHeader {
    fn new(buffer: Bytes) -> Self {
        HTTPHeader {
            buffer,
            method_start: 0,
            method_end: 0,
            path_start: 0,
            path_end: 0,
            protocol_start: 0,
            protocol_end: 0,
        }
    }

    fn method(&self) -> &str {
        std::str::from_utf8(&self.buffer[self.method_start..self.method_end]).unwrap()
    }

    fn path(&self) -> &str {
        std::str::from_utf8(&self.buffer[self.path_start..self.path_end]).unwrap()
    }

    fn protocol(&self) -> &str {
        std::str::from_utf8(&self.buffer[self.protocol_start..self.protocol_end]).unwrap()
    }
}

enum RequestLineState {
    Start,
    Method,
    Path,
    Protocol,
    End,
}

enum ParseResult {
    Again,
    Ok,
    Complete,
    Error,
}

fn parse_http_request_line(buf: Bytes) -> HTTPHeader{
    let mut cursor = Cursor::new(buf.clone());
    let mut header = HTTPHeader::new(buf);

    let mut state = RequestLineState::Start;
    loop {
        let result = match state {
            RequestLineState::Start => {
                let result = parse_start(&mut cursor, &mut header);
                if let ParseResult::Ok = result {
                    state = RequestLineState::Method;
                }
                result
            },
            RequestLineState::Method => {
                let result = parse_method(&mut cursor, &mut header);
                if let ParseResult::Ok = result {
                    state = RequestLineState::Path;
                }
                result
            },
            RequestLineState::Path => {
                let result = parse_path(&mut cursor, &mut header);
                if let ParseResult::Ok = result {
                    state = RequestLineState::Protocol;
                }
                result
            },
            RequestLineState::Protocol => {
                let result: ParseResult = parse_protocol(&mut cursor, &mut header);
                if let ParseResult::Ok = result {
                    state = RequestLineState::End;
                }
                result
            },
            RequestLineState::End => {
                let result: ParseResult = parse_end(&mut cursor, &mut header);
                if let ParseResult::Ok = result {
                    ParseResult::Complete
                } else {
                    result
                }
            }
        };

        if let ParseResult::Complete = result {
            break;
        }
        if let ParseResult::Error = result {
            panic!("Error");
        }
    }
    header
}

fn parse_start(c: &mut Cursor<Bytes>, header :&mut HTTPHeader) -> ParseResult {
    let mut byte = [0; 1];

    loop {
        let size = match c.read(&mut byte) {
            Ok(size) => size,
            Err(_) => {
                return ParseResult::Error;
            }
        };
        if size == 0 {
            return ParseResult::Again;
        } 

        if byte[0] != b'\r' && byte[0] != b'\n' {
            header.method_start = c.position() as usize - 1;
            return ParseResult::Ok;
        }
    }
}


fn parse_method(c: &mut Cursor<Bytes>, header :&mut HTTPHeader) -> ParseResult {
    let mut byte = [0; 1];
    loop {
        let size = match c.read(&mut byte) {
            Ok(size) => size,
            Err(_) => {
                return ParseResult::Error;
            }
        };
        if size == 0 {
            return ParseResult::Again;
        }
        if byte[0] == b' ' {
            header.method_end = c.position() as usize - 1;
            header.path_start = c.position() as usize;
            break;
        }
    }
    ParseResult::Ok
}

fn parse_path(c: &mut Cursor<Bytes>, header :&mut HTTPHeader) -> ParseResult {
    let mut byte = [0; 1];
    loop {
        let size = match c.read(&mut byte) {
            Ok(size) => size,
            Err(_) => {
                return ParseResult::Error;
            }
        };
        if size == 0 {
            return ParseResult::Again;
        }
        if byte[0] == b' ' {
            header.path_end = c.position() as usize - 1;
            header.protocol_start = c.position() as usize;
            break;
        }
    }
    ParseResult::Ok
}


fn parse_protocol(c: &mut Cursor<Bytes>, header :&mut HTTPHeader) -> ParseResult {
    let mut byte = [0; 1];
    loop {
        let size = match c.read(&mut byte) {
            Ok(size) => size,
            Err(_) => {
                return ParseResult::Error;
            }
        };
        if size == 0 {
            return ParseResult::Again;
        }
        let offset = c.position() as usize - header.protocol_start - 1;
        match offset {
            0 => {
                if byte[0] != b'H' {
                    return ParseResult::Error;
                }
            },
            1 => {
                if byte[0] != b'T' {
                    return ParseResult::Error;
                }
            },
            2 => {
                if byte[0] != b'T' {
                    return ParseResult::Error;
                }
            },
            3 => {
                if byte[0] != b'P' {
                    return ParseResult::Error;
                }
            },
            4 => {
                if byte[0] != b'/' {
                    return ParseResult::Error;
                }
            },
            5 => {
                if byte[0] != b'1' {
                    return ParseResult::Error;
                }
            },
            6 => {
                if byte[0] != b'.' {
                    return ParseResult::Error;
                }
            },
            7 => {
                if byte[0] != b'1' {
                    return ParseResult::Error;
                }
                // Success to parse
                header.protocol_end = c.position() as usize;
                return ParseResult::Ok;
            },
            _ => {
                return ParseResult::Error;
            }
        }
    }
}


/// CR LF またｈは LF で終わることを確認する
fn parse_end(c: &mut Cursor<Bytes>, header :&mut HTTPHeader) -> ParseResult {
    let mut byte = [0; 1]; 
    let size = match c.read(&mut byte) {
        Ok(size) => size,
        Err(_) => {
            return ParseResult::Error;
        }
    };
    if size == 0 {
        return ParseResult::Again;
    } 

    if byte[0] == b'\n' {
        return ParseResult::Ok;
    }
    if byte[0] == b'\r' {
        let size = match c.read(&mut byte) {
            Ok(size) => size,
            Err(_) => {
                return ParseResult::Error;
            }
        };
        if size == 0 {
            return ParseResult::Again;
        } 
        if byte[0] == b'\n' {
            return ParseResult::Ok;
        }
        return ParseResult::Error;
    } 
    ParseResult::Error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_request_for_root() {
        let buf = Bytes::from("GET / HTTP/1.1\r\n");
        let header = parse_http_request_line(buf);
        assert_eq!(header.method(), "GET");
        assert_eq!(header.path(), "/");
        assert_eq!(header.protocol(), "HTTP/1.1");
    }

    #[test]
    fn get_request_for_root_lf() {
        let buf = Bytes::from("GET / HTTP/1.1\n");
        let header = parse_http_request_line(buf);
        assert_eq!(header.method(), "GET");
        assert_eq!(header.path(), "/");
        assert_eq!(header.protocol(), "HTTP/1.1");
    }


    #[test]
    fn get_request_for_root_with_head_crlf() {
        let buf = Bytes::from("\r\n\rGET / HTTP/1.1\n");
        let header = parse_http_request_line(buf);
        assert_eq!(header.method(), "GET");
        assert_eq!(header.path(), "/");
        assert_eq!(header.protocol(), "HTTP/1.1");
    }
}