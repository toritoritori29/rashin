use bytes::Bytes;
use std::io::{Cursor, Read};

use super::http_interface::{HTTPHeader, ParseResult};
use super::parse_utility::{read_byte, ReadResult};

#[derive(Clone, Debug)]
pub enum RequestLineState {
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
pub fn parse_http_request_line<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    header: &mut HTTPHeader,
) -> ParseResult<RequestLineState> {
    let mut state = RequestLineState::Start;
    loop {
        let result = match state {
            RequestLineState::Start => {
                let result = parse_start(cursor, header);
                if let ParseResult::Ok(next_state) = &result {
                    state = next_state.clone();
                }
                result
            }
            RequestLineState::Method => {
                let result = parse_method(cursor, header);
                if let ParseResult::Ok(next_state) = &result {
                    state = next_state.clone();
                }
                result
            }
            RequestLineState::Path => {
                let result = parse_path(cursor, header);
                if let ParseResult::Ok(next_state) = &result {
                    state = next_state.clone();
                }
                result
            }
            RequestLineState::Protocol => {
                let result = parse_protocol(cursor, header);
                if let ParseResult::Ok(next_state) = &result {
                    state = next_state.clone();
                }
                result
            }
            RequestLineState::End => {
                let result = parse_end(cursor, header);
                result
            }
        };

        match result {
            ParseResult::Again => {
                return ParseResult::Again;
            }
            ParseResult::Error => {
                return ParseResult::Error;
            }
            ParseResult::Complete => {
                return ParseResult::Complete;
            }
            ParseResult::Ok(_) => {
                continue;
            }
        }
    }
}

fn parse_start<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    header: &mut HTTPHeader,
) -> ParseResult<RequestLineState> {
    loop {
        match read_byte(cursor) {
            ReadResult::Ok(c) => {
                if c == b'\r' || c == b'\n' {
                    continue;
                } else {
                    header.method_start = cursor.position() as usize - 1;
                    return ParseResult::Ok(RequestLineState::Method);
                }
            }
            ReadResult::Again => {
                return ParseResult::Again;
            }
            ReadResult::Err => return ParseResult::Error,
        }
    }
}

fn parse_method<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    header: &mut HTTPHeader,
) -> ParseResult<RequestLineState> {
    loop {
        match read_byte(cursor) {
            ReadResult::Ok(c) => {
                if c == b' ' {
                    header.method_end = cursor.position() as usize - 1;
                    header.path_start = cursor.position() as usize;
                    return ParseResult::Ok(RequestLineState::Path);
                }
            }
            ReadResult::Again => {
                return ParseResult::Again;
            }
            ReadResult::Err => {
                return ParseResult::Error;
            }
        }
    }
}

fn parse_path<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    header: &mut HTTPHeader,
) -> ParseResult<RequestLineState> {
    loop {
        match read_byte(cursor) {
            ReadResult::Ok(c) => {
                if c == b' ' {
                    header.path_end = cursor.position() as usize - 1;
                    header.protocol_start = cursor.position() as usize;
                    return ParseResult::Ok(RequestLineState::Protocol);
                }
                if !c.is_ascii_graphic() {
                    return ParseResult::Error;
                }
            }
            ReadResult::Again => {
                return ParseResult::Again;
            }
            ReadResult::Err => {
                return ParseResult::Error;
            }
        }
    }
}

fn parse_protocol<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    header: &mut HTTPHeader,
) -> ParseResult<RequestLineState> {
    loop {
        let read_result = read_byte(cursor);
        let c = match read_result {
            ReadResult::Ok(c) => c,
            ReadResult::Again => {
                return ParseResult::Again;
            }
            ReadResult::Err => {
                return ParseResult::Error;
            }
        };

        let offset = cursor.position() as usize - header.protocol_start - 1;
        match offset {
            0 => {
                if c != b'H' {
                    return ParseResult::Error;
                }
            }
            1 => {
                if c != b'T' {
                    return ParseResult::Error;
                }
            }
            2 => {
                if c != b'T' {
                    return ParseResult::Error;
                }
            }
            3 => {
                if c != b'P' {
                    return ParseResult::Error;
                }
            }
            4 => {
                if c != b'/' {
                    return ParseResult::Error;
                }
            }
            5 => {
                if c != b'1' {
                    return ParseResult::Error;
                }
            }
            6 => {
                if c != b'.' {
                    return ParseResult::Error;
                }
            }
            7 => {
                if c != b'1' {
                    return ParseResult::Error;
                }
                // Success to parse
                header.protocol_end = cursor.position() as usize;
                return ParseResult::Ok(RequestLineState::End);
            }
            _ => {
                return ParseResult::Error;
            }
        }
    }
}

/// CR LF またｈは LF で終わることを確認する
fn parse_end<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    header: &mut HTTPHeader,
) -> ParseResult<RequestLineState> {
    let read_result = read_byte(cursor);
    let c1 = match read_result {
        ReadResult::Ok(c) => c,
        ReadResult::Again => {
            return ParseResult::Again;
        }
        ReadResult::Err => {
            return ParseResult::Error;
        }
    };

    if c1 == b'\n' {
        return ParseResult::Complete;
    }
    if c1 == b'\r' {
        let c2 = match read_byte(cursor) {
            ReadResult::Ok(c) => c,
            ReadResult::Again => {
                return ParseResult::Again;
            }
            ReadResult::Err => {
                return ParseResult::Error;
            }
        };
        if c2 == b'\n' {
            return ParseResult::Complete;
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
        let mut cursor = Cursor::new(&buf);
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&mut cursor, &mut header);
        assert!(matches!(result, ParseResult::Complete));
        assert_eq!(header.method(&buf), "GET");
        assert_eq!(header.path(&buf), "/");
        assert_eq!(header.protocol(&buf), "HTTP/1.1");
    }

    #[test]
    fn get_request_for_index_html() {
        let buf = Bytes::from("GET /index.html HTTP/1.1\r\n");
        let mut cursor = Cursor::new(&buf);
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&mut cursor, &mut header);
        assert!(matches!(result, ParseResult::Complete));
        assert_eq!(header.method(&buf), "GET");
        assert_eq!(header.path(&buf), "/index.html");
        assert_eq!(header.protocol(&buf), "HTTP/1.1");
    }

    #[test]
    fn get_request_for_root_lf() {
        let buf = Bytes::from("GET / HTTP/1.1\n");
        let mut cursor = Cursor::new(&buf);
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&mut cursor, &mut header);
        assert!(matches!(result, ParseResult::Complete));
        assert_eq!(header.method(&buf), "GET");
        assert_eq!(header.path(&buf), "/");
        assert_eq!(header.protocol(&buf), "HTTP/1.1");
    }

    #[test]
    fn get_request_for_root_with_head_crlf() {
        let buf = Bytes::from("\r\n\rGET / HTTP/1.1\n");
        let mut cursor = Cursor::new(&buf);
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&mut cursor, &mut header);
        assert!(matches!(result, ParseResult::Complete));
        assert_eq!(header.method(&buf), "GET");
        assert_eq!(header.path(&buf), "/");
        assert_eq!(header.protocol(&buf), "HTTP/1.1");
    }

    #[test]
    fn invalid_space_request_should_failed() {
        let buf = Bytes::from("GET   /   HTTP/1.1\n");
        let mut cursor = Cursor::new(&buf);
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&mut cursor, &mut header);
        assert!(matches!(result, ParseResult::Error));
    }

    #[test]
    fn no_path_request_should_failed() {
        let buf = Bytes::from("GET HTTP/1.1\n");
        let mut cursor = Cursor::new(&buf);
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&mut cursor, &mut header);
        assert!(matches!(result, ParseResult::Error));
    }

    #[test]
    fn unknown_protocol_should_failed() {
        let buf = Bytes::from("GET / SMTP/1.1\n");
        let mut cursor = Cursor::new(&buf);
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&mut cursor, &mut header);
        assert!(matches!(result, ParseResult::Error));
    }

    #[test]
    fn unnecessary_suffix_should_failed() {
        let buf = Bytes::from("GET / HTTP/1.1xxx\n");
        let mut cursor = Cursor::new(&buf);
        let mut header = HTTPHeader::new();
        let result = parse_http_request_line(&mut cursor, &mut header);
        assert!(matches!(result, ParseResult::Error));
    }
}
