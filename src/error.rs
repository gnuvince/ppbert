use std::fmt;
use std::result;
use std::error::Error;

#[derive(Debug)]
pub enum BertError {
    InvalidMagicNumber,
    InvalidTag,
    InvalidFloat,
    InvalidUTF8Atom,
    EOF
}

pub type Result<T> = result::Result<T, BertError>;


impl fmt::Display for BertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}


impl Error for BertError {
    fn description(&self) -> &str {
        use self::BertError::*;
        match *self {
            InvalidMagicNumber => "invalid magic number",
            InvalidTag => "invalid tag",
            InvalidFloat => "invalid float",
            InvalidUTF8Atom => "utf8 atom is not correctly encoded",
            EOF => "no more data is available",
        }
    }
}
