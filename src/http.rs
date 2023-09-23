pub mod http_interface;
pub mod parse_request_header;
pub mod parse_request_line;
mod parse_utility;

#[cfg(test)]
mod parse_http_header {
    use super::http_interface::*;
    use super::parse_request_header::*;
    use super::parse_request_line::*;
    use std::io::Cursor;

    #[test]
    fn parse_host_header_successfully() {
        let buf = "\
        GET / HTTP/1.1\r\n\
        Host: localhost:8080\r\n\
        "
        .as_bytes()
        .to_vec();
        let mut cursor = Cursor::new(&buf);

        let mut http_header = HTTPHeader::new();
        {
            let result = parse_http_request_line(&mut cursor, &mut http_header, RequestLineState::Start);
            assert!(matches!(result, ParseResult::Complete));
            assert_eq!(http_header.method(&buf), "GET");
        }

        {
            let mut field = Field::new();
            let result = parse_http_request_header(&mut cursor, &mut field);
            assert!(matches!(result, ParseResult::Complete));
            assert_eq!(field.name(&buf), "Host");
            assert_eq!(field.value(&buf), "localhost:8080");
        }
    }
}
