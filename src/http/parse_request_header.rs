use std::io::{Cursor, Read};

use super::http_interface::{Field, ParseResult, HTTPHeader};
use super::parse_utility::{is_tchar, is_vchar, read_byte, ReadResult};

#[derive(Clone, Debug)]
pub enum RequestHeaderState {
    Start,
    FieldName,
    OWS1,
    FieldValue,
    OWS2,
    End,
}

pub fn parse_http_request_header<'a, T: AsRef<[u8]>>(
    cursor: &'a mut Cursor<T>,
    field: &mut Field,
) -> ParseResult<RequestHeaderState> {
    let mut state = RequestHeaderState::Start;

    loop {
        let result = match state {
            RequestHeaderState::Start => parse_start(cursor, field),
            RequestHeaderState::FieldName => parse_name(cursor, field),
            RequestHeaderState::OWS1 => parse_ows_before_value(cursor, field),
            RequestHeaderState::FieldValue => parse_field_value(cursor, field),
            RequestHeaderState::OWS2 => parse_ows_after_value(cursor, field),
            RequestHeaderState::End => parse_ows_after_value(cursor, field),
        };

        match result {
            ParseResult::Ok(next_state) => {
                state = next_state.clone();
            }
            ParseResult::Again(state) => return ParseResult::Again(state),
            ParseResult::Complete => return ParseResult::Complete,
            ParseResult::Error => return ParseResult::Error,
        }
    }
}

/// ヘッダー行の冒頭を読み込む。
/// * 読み込んだ文字が有効なfield-nameの文字であればfield-nameのパースに遷移する。
///
fn parse_start<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    field: &mut Field,
) -> ParseResult<RequestHeaderState> {
    let read_result = read_byte(cursor);
    match read_result {
        ReadResult::Ok(b'\r') => {
            field.is_separator = true;
            return ParseResult::Ok(RequestHeaderState::End);
        }
        ReadResult::Ok(b'\n') => {
            field.is_separator = true;
            return ParseResult::Complete;
        }
        ReadResult::Ok(c) => {
            if !is_tchar(c) {
                return ParseResult::Error;
            }
            field.name_start = cursor.position() as usize - 1;
            return ParseResult::Ok(RequestHeaderState::FieldName);
        }
        ReadResult::Again => {
            return ParseResult::Again(RequestHeaderState::Start);
        }
        ReadResult::Err => {
            return ParseResult::Error;
        }
    };
}

// field-nameをパースする。
fn parse_name<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    field: &mut Field,
) -> ParseResult<RequestHeaderState> {
    loop {
        let read_result = read_byte(cursor);
        match read_result {
            ReadResult::Ok(b':') => {
                field.name_end = cursor.position() as usize - 1;
                return ParseResult::Ok(RequestHeaderState::OWS1);
            }
            ReadResult::Ok(c) => {
                if !is_tchar(c) {
                    return ParseResult::Error;
                }
            }
            ReadResult::Again => {
                return ParseResult::Again(RequestHeaderState::FieldName);
            }
            ReadResult::Err => {
                return ParseResult::Error;
            }
        };
    }
}

fn parse_ows_before_value<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    field: &mut Field,
) -> ParseResult<RequestHeaderState> {
    loop {
        let read_result = read_byte(cursor);
        match read_result {
            ReadResult::Ok(b' ') => {
                continue;
            }
            ReadResult::Ok(c) => {
                if !is_vchar(c) {
                    return ParseResult::Error;
                }
                field.value_start = cursor.position() as usize - 1;
                return ParseResult::Ok(RequestHeaderState::FieldValue);
            }
            ReadResult::Again => {
                return ParseResult::Again(RequestHeaderState::OWS1);
            }
            ReadResult::Err => {
                return ParseResult::Error;
            }
        };
    }
}

/// field-valueをパースする。
/// field-valueはRFC9110において以下のように定義されている。
///
/// field-content = field-vchar [ 1*( SP / HTAB / field-vchar ) field-vchar ]
/// field-vchar = VCHAR / obs-text
/// ただしvcharはSection2.1に記載のある表示可能な文字である。
/// 
/// References:
/// https://www.rfc-editor.org/rfc/rfc9110#name-syntax-notation
fn parse_field_value<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    field: &mut Field,
) -> ParseResult<RequestHeaderState> {
    let mut prev_ws = false;
    loop {
        let read_result = read_byte(cursor);
        match read_result {
            ReadResult::Ok(b'\r') => {
                field.value_end = cursor.position() as usize - 1;
                return ParseResult::Ok(RequestHeaderState::End);
            }
            ReadResult::Ok(b'\n') => {
                field.value_end = cursor.position() as usize - 1;
                return ParseResult::Complete;
            }
            ReadResult::Ok(b' ') | ReadResult::Ok(b'\t') => {
                if prev_ws {
                    return ParseResult::Ok(RequestHeaderState::OWS2);
                } else {
                    field.value_end = cursor.position() as usize - 1;
                    prev_ws = true;
                }
            }
            ReadResult::Ok(c) => {
                if is_vchar(c) {
                    prev_ws = false;
                } else {
                    return ParseResult::Error;
                }
            }
            ReadResult::Again => {
                return ParseResult::Again(RequestHeaderState::FieldValue);
            }
            ReadResult::Err => {
                return ParseResult::Error;
            }
        };
    }
}

fn parse_ows_after_value<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    field: &mut Field,
) -> ParseResult<RequestHeaderState> {
    loop {
        let read_result = read_byte(cursor);
        match read_result {
            ReadResult::Ok(b' ') => {
                continue;
            }
            ReadResult::Ok(c) => {
                if c == b' ' {
                    continue;
                }
                if c == b'\r' {
                    return ParseResult::Ok(RequestHeaderState::End);
                }
                if c == b'\n' {
                    return ParseResult::Complete;
                }
                return ParseResult::Error;
            }
            ReadResult::Again => {
                return ParseResult::Again(RequestHeaderState::OWS2);
            }
            ReadResult::Err => {
                return ParseResult::Error;
            }
        };
    }
}

fn parse_end_lf<T: AsRef<[u8]>>(
    cursor: &mut Cursor<&T>,
    field: &mut Field,
) -> ParseResult<RequestHeaderState> {
    let read_result = read_byte(cursor);
    match read_result {
        ReadResult::Ok(b'\n') => {
            return ParseResult::Complete;
        }
        ReadResult::Ok(_) => {
            return ParseResult::Error;
        }
        ReadResult::Again => {
            return ParseResult::Again(RequestHeaderState::End);
        }
        ReadResult::Err => {
            return ParseResult::Error;
        }
    };
}


pub fn process_reserved_header(http_header: &mut HTTPHeader, field_name: &str, field_value: &str) {
    match field_name {
        _ => {
            log::debug!(
                "Field: {} = {}", field_name, field_value
            );
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn parse_host_header_successfully() {
        let mut field = Field::new();
        let buf = Bytes::from("Host: localhost:8080\r\n");
        let mut cursor = Cursor::new(&buf);
        let result = parse_http_request_header(&mut cursor, &mut field);
        assert!(matches!(result, ParseResult::Complete));
        assert_eq!(field.name(&buf), "Host");
        assert_eq!(field.value(&buf), "localhost:8080");
    }

    #[test]
    fn parse_header_with_ows_successfully() {
        let mut field = Field::new();
        let buf = Bytes::from("Host:     localhost:8080      \r\n");
        let mut cursor = Cursor::new(&buf);
        let result = parse_http_request_header(&mut cursor, &mut field);

        assert!(matches!(result, ParseResult::Complete));
        assert_eq!(field.name(&buf), "Host");
        assert_eq!(field.value(&buf), "localhost:8080");
    }

    #[test]
    fn parse_header_end_with_lf_successfully() {
        let mut field = Field::new();
        let buf = Bytes::from("Host: localhost:8080\n");
        let mut cursor = &mut Cursor::new(&buf);
        let result = parse_http_request_header(&mut cursor, &mut field);
        assert!(matches!(result, ParseResult::Complete));
        assert_eq!(field.name(&buf), "Host");
        assert_eq!(field.value(&buf), "localhost:8080");
    }

    #[test]
    fn parse_consecutive_headers_successfully() {
        let mut field = Field::new();
        let buf = Bytes::from("Host: localhost:8080\r\nContentType: text-html\r\n");
        let mut cursor = Cursor::new(&buf);

        let result1 = parse_http_request_header(&mut cursor, &mut field);
        assert!(matches!(result1, ParseResult::Complete));
        assert_eq!(field.name(&buf), "Host");
        assert_eq!(field.value(&buf), "localhost:8080");

        let result2 = parse_http_request_header(&mut cursor, &mut field);
        assert!(matches!(result2, ParseResult::Complete));
        assert_eq!(field.name(&buf), "ContentType");
        assert_eq!(field.value(&buf), "text-html");
    }

    #[test]
    fn parse_paused_input_successfully() {
        let mut field = Field::new();
        let mut buf = "Host: local".as_bytes().to_vec();
        let mut cursor = Cursor::new(&mut buf);

        let result1 = parse_http_request_header(&mut cursor, &mut field);
        assert!(matches!(result1, ParseResult::Again(RequestHeaderState::FieldValue)));
        assert_eq!(field.name(cursor.get_ref()), "Host");
    }

    #[test]
    fn parse_empty_line_crlf() {
        let mut field = Field::new();
        let buf = Bytes::from("\r\n");
        let mut cursor = Cursor::new(&buf);
        let result = parse_http_request_header(&mut cursor, &mut field);
        assert!(field.is_separator);
        assert!(matches!(result, ParseResult::Complete));
    }

    #[test]
    fn parse_empty_line_lf() {
        let mut field = Field::new();
        let buf = Bytes::from("\n");
        let mut cursor = Cursor::new(&buf);
        let result = parse_http_request_header(&mut cursor, &mut field);
        assert!(field.is_separator);
        assert!(matches!(result, ParseResult::Complete));
    }
}

