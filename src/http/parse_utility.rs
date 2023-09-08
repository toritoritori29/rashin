use std::io::{Cursor, Read};

pub enum ReadResult {
    Ok(u8),
    Again,
    Err,
}

pub fn read_byte<T: AsRef<[u8]>>(buf: &mut Cursor<T>) -> ReadResult {
    let mut b = [0; 1];
    let size = buf.read(&mut b);

    match size {
        Ok(0) => ReadResult::Again,
        Ok(1) => ReadResult::Ok(b[0]),
        _ => ReadResult::Err,
    }
}
