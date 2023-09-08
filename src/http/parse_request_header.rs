

use bytes::Bytes;
use std::io::{Cursor, Read};

use super::http_interface::{HTTPHeader, ParseResult};
use super::parse_utility::{read_byte, ReadResult};

enum RequestLineState {
    Start,
    Method,
    Path,
    Protocol,
    End,
}

/// RequestLineの実装
/// リエントラントにするよう実装する。
/// 
/// TODO: read周りが冗長なのでユーティリティ関数を作る
/// TODO: HTTP1.1しかパースできないのでもう少し汎用的にする
/// TODO: パスの中身をしっかり検証していない
fn parse_http_request_line<'a>(buf: &'a Bytes, header: &mut HTTPHeader) -> ParseResult {
    let mut cursor = Cursor::new(buf);
    let mut state = RequestLineState::Start;
    loop {
        let result = match state {
            RequestLineState::Start => {
                let result = parse_start(&mut cursor, header);
                if let ParseResult::Ok = result {
                    state = RequestLineState::Method;
                }
                result
            },
            RequestLineState::Method => {
                let result = parse_method(&mut cursor, header);
                if let ParseResult::Ok = result {
                    state = RequestLineState::Path;
                }
                result
            },
            RequestLineState::Path => {
                let result = parse_path(&mut cursor, header);
                if let ParseResult::Ok = result {
                    state = RequestLineState::Protocol;
                }
                result
            },
            RequestLineState::Protocol => {
                let result: ParseResult = parse_protocol(&mut cursor, header);
                if let ParseResult::Ok = result {
                    state = RequestLineState::End;
                }
                result
            },
            RequestLineState::End => {
                let result: ParseResult = parse_end(&mut cursor, header);
                if let ParseResult::Ok = result {
                    ParseResult::Complete
                } else {
                    result
                }
            }
        };

        match result {
            ParseResult::Again => {
                return ParseResult::Again;
            },
            ParseResult::Error => {
                return ParseResult::Error;
            },
            ParseResult::Complete => {
                return ParseResult::Complete;
            },
            ParseResult::Ok => {
                continue;
            }
        }
    }
}

fn parse_start(cursor: &mut Cursor<&Bytes>, header :&mut HTTPHeader) -> ParseResult {
    loop {
        match read_byte(cursor) {
            ReadResult::Ok(c) => {
                if c == b'\r' || c == b'\n' {
                    continue;
                } else {
                    header.method_start = cursor.position() as usize - 1;
                    return ParseResult::Ok;
                }
           },
            ReadResult::Again => {
                return ParseResult::Again;
            },
            ReadResult::Err => {
                return ParseResult::Error
            }
        }
    }
}


fn parse_method(c: &mut Cursor<&Bytes>, header :&mut HTTPHeader) -> ParseResult {
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

fn parse_path(c: &mut Cursor<&Bytes>, header :&mut HTTPHeader) -> ParseResult {
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


fn parse_protocol(c: &mut Cursor<&Bytes>, header :&mut HTTPHeader) -> ParseResult {
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
fn parse_end(c: &mut Cursor<&Bytes>, header :&mut HTTPHeader) -> ParseResult {
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
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&buf, &mut header);
        assert!(matches!(result, ParseResult::Complete));
        assert_eq!(header.method(&buf), "GET");
        assert_eq!(header.path(&buf), "/");
        assert_eq!(header.protocol(&buf), "HTTP/1.1");
    }

    #[test]
    fn get_request_for_index_html() {
        let buf = Bytes::from("GET /index.html HTTP/1.1\r\n");
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&buf, &mut header);
        assert!(matches!(result, ParseResult::Complete));
        assert_eq!(header.method(&buf), "GET");
        assert_eq!(header.path(&buf), "/index.html");
        assert_eq!(header.protocol(&buf), "HTTP/1.1");
    }

    #[test]
    fn get_request_for_root_lf() {
        let buf = Bytes::from("GET / HTTP/1.1\n");
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&buf, &mut header);
        assert!(matches!(result, ParseResult::Complete));
        assert_eq!(header.method(&buf), "GET");
        assert_eq!(header.path(&buf), "/");
        assert_eq!(header.protocol(&buf), "HTTP/1.1");
    }


    #[test]
    fn get_request_for_root_with_head_crlf() {
        let buf = Bytes::from("\r\n\rGET / HTTP/1.1\n");
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&buf, &mut header);
        assert!(matches!(result, ParseResult::Complete));
        assert_eq!(header.method(&buf), "GET");
        assert_eq!(header.path(&buf), "/");
        assert_eq!(header.protocol(&buf), "HTTP/1.1");
    }
}