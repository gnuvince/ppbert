use std::fmt;
use std::result;
use std::error::Error;
use std::io;

use crate::prelude::*;

#[derive(Debug)]
pub enum BertError {
    // io errors
    IoError(io::Error),

    // parsing errors
    InvalidMagicNumber { offset: usize, actual: u8 },
    InvalidTag(usize, u8),
    InvalidFloat(usize),
    InvalidUTF8Atom(usize),
    InvalidLatin1Atom(usize),
    VarintTooLarge(usize),
    NotEnoughData { offset: usize, needed: usize, available: usize },
    InvalidDiskLogMagic { offset: usize, actual: u32 },
    InvalidDiskLogTermMagic { offset: usize, actual: u32 },
    InvalidDiskLogOpenedStatus { offset: usize, actual: u32 },
}

impl fmt::Display for BertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::BertError::*;
        match *self {
            IoError(ref io_err) =>
                write!(f, "{}", io_err),
            InvalidMagicNumber { offset, actual } =>
                write!(f, "invalid magic number at offset {}: expected 0x{:02x}, found 0x{:02x}",
                       offset, BERT_MAGIC_NUMBER, actual),
            InvalidTag(offset, byte) =>
                write!(f, "invalid tag at offset {}: 0x{:02x}", offset, byte),
            InvalidFloat(offset) =>
                write!(f, "invalid float at offset {}", offset),
            InvalidUTF8Atom(offset) =>
                write!(f, "UTF-8 atom is not correctly encoded at offset {}", offset),
            InvalidLatin1Atom(offset) =>
                write!(f, "Latin-1 atom is not correctly encoded at offset {}", offset),
            VarintTooLarge(offset) =>
                write!(f, "varint is too large (greater than 2^64-1) at offset {}", offset),
            NotEnoughData { needed, available, offset } =>
                write!(f, "no enough data available at offset {}: needed {} bytes, only {} remaining",
                       offset, needed, available),
            InvalidDiskLogMagic { offset, actual } =>
                write!(f, "invalid disk_log magic at {}: expected 0x{:08x}, found 0x{:08x}",
                       offset, DISK_LOG_MAGIC, actual),
            InvalidDiskLogTermMagic { offset, actual } =>
                write!(f, "invalid disk_log term magic at offset {}: expected 0x{:08x}, found 0x{:08x}",
                       offset, DISK_LOG_TERM_MAGIC, actual),
            InvalidDiskLogOpenedStatus { offset, actual } =>
                write!(f, "invalid disk_log opened status at offset {}: expected 0x{:08x}, found 0x{:08x}",
                       offset, DISK_LOG_OPENED, actual),
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
