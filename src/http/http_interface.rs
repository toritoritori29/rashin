
pub struct HTTPHeader {
    pub method_start: usize,
    pub method_end: usize,
    pub path_start: usize,
    pub path_end: usize,
    pub protocol_start: usize,
    pub protocol_end: usize,

    pub field_size: usize,
    pub fields: Vec<Field>,
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
            field_size: 0,
            fields: Vec::new(),
        }
    }

    pub fn method<'a, T: AsRef<[u8]>>(&self, buffer: &'a T) -> &'a str {
        std::str::from_utf8(&buffer.as_ref()[self.method_start..self.method_end]).unwrap()
    }

    pub fn path<'a, T: AsRef<[u8]>>(&self, buffer: &'a T) -> &'a str {
        std::str::from_utf8(&buffer.as_ref()[self.path_start..self.path_end]).unwrap()
    }

    pub fn protocol<'a, T: AsRef<[u8]>>(&self, buffer: &'a T) -> &'a str {
        std::str::from_utf8(&buffer.as_ref()[self.protocol_start..self.protocol_end]).unwrap()
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
        self.field_size += 1;
    }
}

pub struct Field {
    pub is_separator: bool,
    pub name_start: usize,
    pub name_end: usize,
    pub value_start: usize,
    pub value_end: usize,
}

impl Field {
    pub fn new() -> Self {
        Field {
            is_separator: false,
            name_start: 0,
            name_end: 0,
            value_start: 0,
            value_end: 0,
        }
    }

    pub fn name<'a, T: AsRef<[u8]>>(&self, buffer: &'a T) -> &'a str {
        std::str::from_utf8(&buffer.as_ref()[self.name_start..self.name_end]).unwrap()
    }

    pub fn value<'a, T: AsRef<[u8]>>(&self, buffer: &'a T) -> &'a str {
        std::str::from_utf8(&buffer.as_ref()[self.value_start..self.value_end]).unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParseResult<T> {
    Again(T),
    Ok(T),
    Complete,
    Error,
}
