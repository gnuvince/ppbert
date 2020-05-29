use std::io;

use byteorder::{BigEndian, WriteBytesExt};
use num_bigint::{Sign};

use crate::prelude::*;

pub struct BertWriter;

impl BertWriter {
    pub fn new() -> Self {
        BertWriter { }
    }

    pub fn write<W: io::Write>(&self, term: &BertTerm, w: &mut W) -> Result<()> {
        w.write_u8(BERT_MAGIC_NUMBER)?;
        self.write_bert(term, w)?;
        return Ok(());
    }

    fn write_bert<W: io::Write>(&self, term: &BertTerm, w: &mut W) -> io::Result<()> {
        match *term {
            BertTerm::Nil => w.write_u8(NIL_EXT),
            BertTerm::Int(n) => {
                if n >= 0 && n < 256 {
                    w.write_u8(SMALL_INTEGER_EXT)?;
                    w.write_u8(n as u8)
                } else {
                    w.write_u8(INTEGER_EXT)?;
                    w.write_i32::<BigEndian>(n)
                }
            }
            BertTerm::BigInt(ref b) => {
                let (sign, bytes) = b.to_bytes_le();
                let len = bytes.len();
                if len < 256 {
                    w.write_u8(SMALL_BIG_EXT)?;
                    w.write_u8(len as u8)?;
                } else {
                    w.write_u8(LARGE_BIG_EXT)?;
                    w.write_u32::<BigEndian>(len as u32)?;
                }
                if sign == Sign::Minus {
                    w.write_u8(1)?;
                } else {
                    w.write_u8(0)?;
                }
                w.write_all(&bytes)
            }
            BertTerm::Float(f) => {
                w.write_u8(NEW_FLOAT_EXT)?;
                w.write_f64::<BigEndian>(f)
            }
            BertTerm::Tuple(ref terms) => {
                let len = terms.len();
                if len < 256 {
                    w.write_u8(SMALL_TUPLE_EXT)?;
                    w.write_u8(len as u8)?;
                } else {
                    w.write_u8(LARGE_TUPLE_EXT)?;
                    w.write_u32::<BigEndian>(len as u32)?;
                }
                for t in terms {
                    self.write_bert(t, w)?;
                }
                Ok(())
            }
            BertTerm::List(ref terms) => {
                let len = terms.len();
                w.write_u8(LIST_EXT)?;
                w.write_u32::<BigEndian>(len as u32)?;
                for t in terms {
                    self.write_bert(t, w)?;
                }
                w.write_u8(NIL_EXT)
            }
            BertTerm::Map(ref keys, ref vals) => {
                let len = keys.len();
                w.write_u8(MAP_EXT)?;
                w.write_u32::<BigEndian>(len as u32)?;
                for (k, v) in keys.iter().zip(vals) {
                    self.write_bert(k, w)?;
                    self.write_bert(v, w)?;
                }
                Ok(())
            }
            BertTerm::Atom(ref chars) => {
                let bytes = chars.as_bytes();
                let len = bytes.len();
                w.write_u8(ATOM_UTF8_EXT)?;
                w.write_u16::<BigEndian>(len as u16)?;
                w.write_all(bytes)
            }
            BertTerm::String(ref bytes) => {
                let len = bytes.len();
                w.write_u8(STRING_EXT)?;
                w.write_u16::<BigEndian>(len as u16)?;
                w.write_all(bytes)
            }
            BertTerm::Binary(ref bytes) => {
                let len = bytes.len();
                w.write_u8(BINARY_EXT)?;
                w.write_u32::<BigEndian>(len as u32)?;
                w.write_all(bytes)
            }
        }
    }
}
