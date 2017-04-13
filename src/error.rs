use std::fmt;
use std::result;
use std::error::Error;

#[derive(Debug)]
pub enum BertError {
    // input errors
    CannotOpenFile,

    // parsing errors
    InvalidMagicNumber(usize),
    InvalidTag(usize, u8),
    InvalidFloat(usize),
    InvalidUTF8Atom(usize),
    InvalidLatin1Atom(usize),
    ExtraData(usize),
    EOF(usize)
}

impl BertError {
    fn offset(&self) -> Option<usize> {
        use self::BertError::*;
        match *self {
            InvalidMagicNumber(offset)
            | InvalidTag(offset, _)
            | InvalidFloat(offset)
            | InvalidUTF8Atom(offset)
            | InvalidLatin1Atom(offset)
            | ExtraData(offset)
            | EOF(offset) => Some(offset),

            _ => None
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
        match self.offset() {
            Some(offset) =>
                write!(f, "{} at offset {}", self.description(), offset)?,
            None =>
                write!(f, "{}", self.description())?
        }
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
            CannotOpenFile => "cannot open file",
            InvalidMagicNumber(_) => "invalid magic number",
            InvalidTag(_, _) => "invalid tag",
            InvalidFloat(_) => "invalid float",
            InvalidUTF8Atom(_) => "UTF-8 atom is not correctly encoded",
            InvalidLatin1Atom(_) => "Latin-1 atom is not correctly encoded",
            ExtraData(_) => "extra data after the BERT term",
            EOF(_) => "no more data is available",
        }
    }
}
