use std::fmt;
use std::result;
use std::error::Error;

#[derive(Debug)]
pub enum BertError {
    InvalidMagicNumber(usize),
    InvalidTag(usize, u8),
    InvalidFloat(usize),
    InvalidUTF8Atom(usize),
    EOF(usize)
}

impl BertError {
    fn offset(&self) -> usize {
        use self::BertError::*;
        match *self {
            InvalidMagicNumber(offset)
                | InvalidTag(offset, _)
                | InvalidFloat(offset)
                | InvalidUTF8Atom(offset)
                | EOF(offset) => offset
        }
    }

    fn extra_info(&self) -> Option<String> {
        use self::BertError::*;
        match *self {
            InvalidTag(_, tag) => Some(format!("{}", tag)),
            _ => None
        }
    }
}

pub type Result<T> = result::Result<T, BertError>;


impl fmt::Display for BertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _ = write!(f, "{} at offset {}",
                       self.description(), self.offset());
        match self.extra_info() {
            None => write!(f, ""),
            Some(ref s) => write!(f, ": {}", s)
        }
    }
}


impl Error for BertError {
    fn description(&self) -> &str {
        use self::BertError::*;
        match *self {
            InvalidMagicNumber(_) => "invalid magic number",
            InvalidTag(_, _) => "invalid tag",
            InvalidFloat(_) => "invalid float",
            InvalidUTF8Atom(_) => "utf8 atom is not correctly encoded",
            EOF(_) => "no more data is available",
        }
    }
}
