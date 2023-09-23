use std::io::{Cursor, Read};

pub enum ReadResult {
    Ok(u8),
    Again,
    Err,
}

pub fn read_byte<T: AsRef<[u8]>>(cursor: &mut Cursor<T>) -> ReadResult {
    let mut b = [0; 1];
    let size = cursor.read(&mut b);

    match size {
        Ok(0) => ReadResult::Again,
        Ok(1) => ReadResult::Ok(b[0]),
        _ => ReadResult::Err,
    }
}

/// Check if the given byte is a tchar.
/// "tchar" is a character type defined in RFC9110.
/// "tchar" is used in field names and so on. For details,
/// please refer to the Appendix of RFC9110.
///
/// tchar = "!" / "#" / "$" / "%" / "&" / "'" / "*" / "+" / "-" / "." /
///         "^" / "_" / "`" / "|" / "~" / DIGIT / ALPHA
/// Reference:
/// https://www.rfc-editor.org/rfc/rfc9110#name-collected-abnf
///
pub fn is_tchar(byte: u8) -> bool {
    if byte.is_ascii_alphanumeric() {
        return true;
    }
    match byte {
        b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' | b'.' | b'^' | b'_'
        | b'`' | b'|' | b'~' => true,
        _ => false,
    }
}

/// Check if the given byte is a tchar.
/// "vchar" is a chracter that can be displayed.
/// Reference:
/// https://www.rfc-editor.org/rfc/rfc9110#name-syntax-notation
pub fn is_vchar(byte: u8) -> bool {
    byte.is_ascii_graphic()
}
