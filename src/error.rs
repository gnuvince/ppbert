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
    VarintTooLarge(usize),
    NotEnoughData { offset: usize, needed: usize, available: usize },
    InvalidDiskLogOpenedStatus(usize),
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
            | VarintTooLarge(offset)
            | InvalidDiskLogOpenedStatus(offset)
            | NotEnoughData { offset, .. } => Some(offset),
            _ => None
        }
    }

    fn extra_info(&self) -> Option<String> {
        use self::BertError::*;
        match *self {
            InvalidTag(_, tag) => Some(format!("{}", tag)),
            NotEnoughData { needed, available, .. } =>
                Some(format!("bytes needed: {}; bytes available: {}", needed, available)),
            _ => None
        }
    }
}

impl fmt::Display for BertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::BertError::*;
        match *self {
            IoError(ref io_err) => write!(f, "{}", io_err)?,
            InvalidMagicNumber(_) => write!(f, "invalid magic number")?,
            InvalidTag(_, _) => write!(f, "invalid tag")?,
            InvalidFloat(_) => write!(f, "invalid float")?,
            InvalidUTF8Atom(_) => write!(f, "UTF-8 atom is not correctly encoded")?,
            InvalidLatin1Atom(_) => write!(f, "Latin-1 atom is not correctly encoded")?,
            VarintTooLarge(_) => write!(f, "varint is too large (greater than 2^64-1)")?,
            NotEnoughData { .. } => write!(f, "no enough data available")?,
            InvalidDiskLogOpenedStatus(_) => write!(f, "invalid file opened status")?,
        }

        match self.offset() {
            Some(offset) => write!(f, " at offset {}", offset)?,
            None => (),
        }
        match self.extra_info() {
            Some(ref s) => write!(f, ": {}", s),
            None => Ok(()),
        }
    }
}

impl Error for BertError {}

pub type Result<T> = result::Result<T, BertError>;

impl From<io::Error> for BertError {
    fn from(io_err: io::Error) -> BertError {
        BertError::IoError(io_err)
    }
}
