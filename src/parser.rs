use crate::prelude::*;

use num_bigint::{BigInt, ToBigInt};
use num_traits::{One, Zero};

use byteorder::{BigEndian, ReadBytesExt};

use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};

pub type ParserNext = fn(&mut BertParser) -> Option<Result<BertTerm>>;

#[derive(Debug)]
pub struct BertParser {
    contents: Vec<u8>,
    pos: usize,
}

impl BertParser {
    pub fn new(contents: Vec<u8>) -> BertParser {
        BertParser {
            contents: contents,
            pos: 0,
        }
    }

    // "Iterators"
    pub fn bert1_next(&mut self) -> Option<Result<BertTerm>> {
        if self.eof() {
            return None;
        }
        let result = self.magic_number().and_then(|_| self.bert_term());
        return Some(result);
    }

    pub fn bert2_next(&mut self) -> Option<Result<BertTerm>> {
        if self.eof() {
            return None;
        }
        let result = self
            .parse_varint()
            .and_then(|_| self.magic_number())
            .and_then(|_| self.bert_term());
        return Some(result);
    }

    pub fn disk_log_next(&mut self) -> Option<Result<BertTerm>> {
        if self.eof() {
            return None;
        }
        let result = self
            .disk_log_magic()
            .and_then(|_| self.disk_log_opened_status())
            .and_then(|_| self.disk_log_term());
        return Some(result);
    }

    // Parsers
    pub fn magic_number(&mut self) -> Result<()> {
        let initial_pos = self.pos;
        let magic = self.eat_u8()?;
        if magic != BERT_MAGIC_NUMBER {
            return Err(BertError::InvalidMagicNumber {
                offset: initial_pos,
                actual: magic,
            });
        }
        return Ok(());
    }

    pub fn disk_log_magic(&mut self) -> Result<()> {
        let initial_pos = self.pos;
        let magic = self.eat_u32_be()?;
        if magic != DISK_LOG_MAGIC {
            return Err(BertError::InvalidDiskLogMagic {
                offset: initial_pos,
                actual: magic,
            });
        }
        return Ok(());
    }

    pub fn disk_log_opened_status(&mut self) -> Result<()> {
        let initial_pos = self.pos;
        let status = self.eat_u32_be()?;
        if status != DISK_LOG_OPENED && status != DISK_LOG_CLOSED {
            return Err(BertError::InvalidDiskLogOpenedStatus {
                offset: initial_pos,
                actual: status,
            });
        }
        return Ok(());
    }

    pub fn disk_log_term(&mut self) -> Result<BertTerm> {
        // XXX(vfoley): should we check that the correct length was read?
        let _len_offset = self.pos;
        let _len = self.eat_u32_be()?;

        let magic_pos = self.pos;
        let magic = self.eat_u32_be()?;
        if magic != DISK_LOG_TERM_MAGIC {
            return Err(BertError::InvalidDiskLogTermMagic {
                offset: magic_pos,
                actual: magic,
            });
        }

        let magic_pos = self.pos;
        let magic = self.eat_u8()?;
        if magic != BERT_MAGIC_NUMBER {
            return Err(BertError::InvalidMagicNumber {
                offset: magic_pos,
                actual: magic,
            });
        }

        return self.bert_term();
    }

    pub fn bert_term(&mut self) -> Result<BertTerm> {
        let initial_pos = self.pos;
        match self.eat_u8()? {
            SMALL_INTEGER_EXT => self.small_integer(),
            INTEGER_EXT => self.integer(),
            FLOAT_EXT => self.old_float(),
            NEW_FLOAT_EXT => self.new_float(),
            ATOM_EXT => {
                let len = self.eat_u16_be()? as usize;
                self.atom(len)
            }
            SMALL_ATOM_EXT => {
                let len = self.eat_u8()? as usize;
                self.atom(len)
            }
            ATOM_UTF8_EXT => {
                let len = self.eat_u16_be()? as usize;
                self.atom_utf8(len)
            }
            SMALL_ATOM_UTF8_EXT => {
                let len = self.eat_u8()? as usize;
                self.atom_utf8(len)
            }
            SMALL_TUPLE_EXT => {
                let len = self.eat_u8()? as usize;
                self.tuple(len)
            }
            LARGE_TUPLE_EXT => {
                let len = self.eat_u32_be()? as usize;
                self.tuple(len)
            }
            NIL_EXT => Ok(BertTerm::Nil),
            LIST_EXT => self.list(),
            STRING_EXT => self.string(),
            BINARY_EXT => self.binary(),
            SMALL_BIG_EXT => {
                let len = self.eat_u8()?;
                self.bigint(len as usize)
            }
            LARGE_BIG_EXT => {
                let len = self.eat_u32_be()?;
                self.bigint(len as usize)
            }
            MAP_EXT => self.map(),
            tag => Err(BertError::InvalidTag(initial_pos, tag)),
        }
    }

    pub fn small_integer(&mut self) -> Result<BertTerm> {
        let b = self.eat_u8()?;
        Ok(BertTerm::Int(b as i32))
    }

    pub fn integer(&mut self) -> Result<BertTerm> {
        let n = self.eat_i32_be()?;
        Ok(BertTerm::Int(n))
    }

    pub fn old_float(&mut self) -> Result<BertTerm> {
        let initial_pos = self.pos;
        let mut s = String::new();
        while !self.eof() && self.peek()? != 0 {
            s.push(self.eat_char()?);
        }

        while !self.eof() && self.peek()? == 0 {
            let _ = self.eat_u8()?;
        }

        s.parse::<f64>()
            .map_err(|_| BertError::InvalidFloat(initial_pos))
            .map(|f| BertTerm::Float(f))
    }

    pub fn new_float(&mut self) -> Result<BertTerm> {
        let raw_bytes = self.eat_u64_be()?;
        let f: f64 = f64::from_bits(raw_bytes);
        Ok(BertTerm::Float(f))
    }

    pub fn atom(&mut self, len: usize) -> Result<BertTerm> {
        let initial_pos = self.pos;
        let bytes: Vec<u8> = self.eat_slice(len)?.to_owned();
        let is_ascii = bytes.iter().all(|byte| *byte < 128);

        // Optimization: ASCII atoms represent the overwhelming
        // majority of use cases of atoms. When we read the bytes
        // of the atom, we record whether they are all ASCII
        // (i.e., small than 128); if it's the case, we don't
        // need to bother with latin-1 decoding. We use an unsafe
        // method because ASCII strings are guaranteed to be valid
        // UTF-8 strings.
        if is_ascii {
            let s = unsafe { String::from_utf8_unchecked(bytes) };
            Ok(BertTerm::Atom(s))
        } else {
            ISO_8859_1
                .decode(&bytes, DecoderTrap::Strict)
                .map(|s| BertTerm::Atom(s))
                .map_err(|_| BertError::InvalidLatin1Atom(initial_pos))
        }
    }

    pub fn atom_utf8(&mut self, len: usize) -> Result<BertTerm> {
        let initial_pos = self.pos;
        let buf: Vec<u8> = self.eat_slice(len)?.to_owned();
        String::from_utf8(buf)
            .map(|s| BertTerm::Atom(s))
            .map_err(|_| BertError::InvalidUTF8Atom(initial_pos))
    }

    pub fn tuple(&mut self, len: usize) -> Result<BertTerm> {
        let mut terms = Vec::with_capacity(len);
        for _ in 0..len {
            terms.push(self.bert_term()?);
        }
        Ok(BertTerm::Tuple(terms))
    }

    pub fn string(&mut self) -> Result<BertTerm> {
        let len = self.eat_u16_be()? as usize;
        let bytes: Vec<u8> = self.eat_slice(len)?.to_owned();
        Ok(BertTerm::String(bytes))
    }

    pub fn binary(&mut self) -> Result<BertTerm> {
        let len = self.eat_u32_be()? as usize;
        let bytes: Vec<u8> = self.eat_slice(len)?.to_owned();
        Ok(BertTerm::Binary(bytes))
    }

    pub fn list(&mut self) -> Result<BertTerm> {
        let len = self.eat_u32_be()?;
        let mut terms = Vec::with_capacity(len as usize + 1);
        for _ in 0..len {
            terms.push(self.bert_term()?);
        }
        let tail = self.bert_term()?;
        match tail {
            BertTerm::Nil => (),
            last_term => {
                terms.push(last_term);
            }
        };
        Ok(BertTerm::List(terms))
    }

    pub fn bigint(&mut self, len: usize) -> Result<BertTerm> {
        let sign = self.eat_u8()?;
        let mut sum: BigInt = Zero::zero();
        let mut pos: BigInt = One::one();
        for _ in 0..len {
            let d = self.eat_u8()?;
            let t = &pos * &(d.to_bigint().unwrap());
            sum = sum + &t;
            pos = pos * (256).to_bigint().unwrap();
        }
        if sign == 1 {
            sum = -sum;
        }
        Ok(BertTerm::BigInt(sum))
    }

    // TODO(vfoley): ensure no duplicate keys
    pub fn map(&mut self) -> Result<BertTerm> {
        let len = self.eat_u32_be()? as usize;
        let mut keys = Vec::with_capacity(len);
        let mut vals = Vec::with_capacity(len);
        for _ in 0..len {
            keys.push(self.bert_term()?);
            vals.push(self.bert_term()?);
        }
        Ok(BertTerm::Map(keys, vals))
    }

    // Low-level parsing methods
    pub fn eof(&self) -> bool {
        self.pos >= self.contents.len()
    }

    pub fn peek(&self) -> Result<u8> {
        if self.eof() {
            return Err(BertError::NotEnoughData {
                offset: self.pos,
                needed: 1,
                available: 0,
            });
        } else {
            return Ok(self.contents[self.pos]);
        }
    }

    pub fn can_read(&self, n: usize) -> bool {
        (self.pos + n) <= self.contents.len()
    }

    pub fn eat_slice(&mut self, len: usize) -> Result<&[u8]> {
        if !self.can_read(len) {
            return Err(BertError::NotEnoughData {
                offset: self.pos,
                needed: len,
                available: self.contents.len() - self.pos,
            });
        }
        let slice = &self.contents[self.pos..self.pos + len];
        self.pos += len;
        return Ok(slice);
    }

    pub fn eat_u8(&mut self) -> Result<u8> {
        let b = self.peek()?;
        self.pos += 1;
        return Ok(b);
    }

    pub fn eat_char(&mut self) -> Result<char> {
        let b = self.eat_u8()?;
        return Ok(b as char);
    }

    pub fn eat_u16_be(&mut self) -> Result<u16> {
        let mut bytes = self.eat_slice(2)?;
        let n = bytes.read_u16::<BigEndian>()?;
        return Ok(n);
    }

    pub fn eat_i32_be(&mut self) -> Result<i32> {
        let mut bytes = self.eat_slice(4)?;
        let n = bytes.read_i32::<BigEndian>()?;
        return Ok(n);
    }

    pub fn eat_u32_be(&mut self) -> Result<u32> {
        let mut bytes = self.eat_slice(4)?;
        let n = bytes.read_u32::<BigEndian>()?;
        return Ok(n);
    }

    pub fn eat_u64_be(&mut self) -> Result<u64> {
        let mut bytes = self.eat_slice(8)?;
        let n = bytes.read_u64::<BigEndian>()?;
        return Ok(n);
    }

    // https://developers.google.com/protocol-buffers/docs/encoding#varints
    pub fn parse_varint(&mut self) -> Result<u64> {
        const MAX_LEN: u64 = 8;
        let start_pos = self.pos;
        let mut i: u64 = 0;
        let mut val: u64 = 0;

        while !self.eof() && i < MAX_LEN {
            let b = self.eat_u8()?;
            val = val | ((b as u64 & 0x7f) << (7 * i));
            if b & 0x80 == 0 {
                break;
            }
            i += 1;
        }

        if i >= MAX_LEN {
            return Err(BertError::VarintTooLarge(start_pos));
        }
        return Ok(val);
    }
}

#[test]
fn test_varint() {
    assert_eq!(1, {
        match BertParser::new(vec![1]).parse_varint() {
            Ok(x) => x,
            Err(_) => u64::max_value(),
        }
    });

    assert_eq!(300, {
        match BertParser::new(vec![0b1010_1100, 0b0000_0010]).parse_varint() {
            Ok(x) => x,
            Err(_) => u64::max_value(),
        }
    });

    assert!(
        BertParser::new(vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f])
            .parse_varint()
            .is_ok()
    );
    assert!(
        BertParser::new(vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x80])
            .parse_varint()
            .is_err()
    );
}
