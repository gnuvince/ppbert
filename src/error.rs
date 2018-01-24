use std::fmt;
use std::result;
use std::error::Error;
use std::io;

#[derive(Debug)]
pub enum BertError {
    // io errors
    IoError(io::Error),

    // parsing errors
    InvalidMagicNumber(usize),
    InvalidTag(usize, u8),
    InvalidFloat(usize),
    InvalidUTF8Atom(usize),
    InvalidLatin1Atom(usize),
    ExtraData(usize),
    VarintTooLarge(usize),
    NotEnoughData { offset: usize, needed: usize, available: usize }
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
            | VarintTooLarge(offset)
            | NotEnoughData { offset, .. } => Some(offset),

            _ => None
        }
    }

    fn extra_info(&self) -> Option<String> {
        use self::BertError::*;
        match *self {
            InvalidTag(_, tag) => Some(format!("{}", tag)),
            NotEnoughData { needed, available, .. } =>
                Some(format!("needed {} bytes, only {} available", needed, available)),
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
            IoError(ref io_err) => io_err.description(),
            InvalidMagicNumber(_) => "invalid magic number",
            InvalidTag(_, _) => "invalid tag",
            InvalidFloat(_) => "invalid float",
            InvalidUTF8Atom(_) => "UTF-8 atom is not correctly encoded",
            InvalidLatin1Atom(_) => "Latin-1 atom is not correctly encoded",
            ExtraData(_) => "extra data after the BERT term",
            VarintTooLarge(_) => "varint is too large (greater than 2^64-1)",
            NotEnoughData { .. } => "no enough data is available",
        }
    }
}


impl From<io::Error> for BertError {
    fn from(io_err: io::Error) -> BertError {
        BertError::IoError(io_err)
    }
}
