use std::mem;

use num::bigint::{self, ToBigInt};
use num::traits::{Zero, One};

use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;

use bertterm::BertTerm;
use error::{Result, BertError};

const BERT_MAGIC_NUMBER: u8 = 131;
const SMALL_INTEGER_EXT: u8 = 97;
const INTEGER_EXT: u8 = 98;
const FLOAT_EXT: u8 = 99;
const ATOM_EXT: u8 = 100;
const SMALL_ATOM_EXT: u8 = 115;
const SMALL_TUPLE_EXT: u8 = 104;
const LARGE_TUPLE_EXT: u8 = 105;
const NIL_EXT: u8 = 106;
const STRING_EXT: u8 = 107;
const LIST_EXT: u8 = 108;
const BINARY_EXT: u8 = 109;
const SMALL_BIG_EXT: u8 = 110;
const LARGE_BIG_EXT: u8 = 111;
const ATOM_UTF8_EXT: u8 = 118;
const SMALL_ATOM_UTF8_EXT: u8 = 119;
const NEW_FLOAT_EXT: u8 = 70;

#[derive(Debug)]
pub struct Parser {
    contents: Vec<u8>,
    pos: usize,
}

impl Parser {
    pub fn new(contents: Vec<u8>) -> Parser {
        Parser { contents: contents, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<BertTerm> {
        let () = self.magic_number()?;
        let term = self.bert_term()?;
        if self.eof() {
            return Ok(term);
        } else {
            return Err(BertError::ExtraData(self.pos));
        }
    }

    // Parsers
    fn magic_number(&mut self) -> Result<()> {
        let offset = self.pos;
        let magic = self.eat_u8()?;
        if magic != BERT_MAGIC_NUMBER {
            return Err(BertError::InvalidMagicNumber(offset));
        } else {
            return Ok(());
        }
    }

    fn bert_term(&mut self) -> Result<BertTerm> {
        let offset = self.pos;
        match self.eat_u8()? {
            SMALL_INTEGER_EXT => { self.small_integer() }
            INTEGER_EXT => { self.integer() }
            FLOAT_EXT => { self.old_float() }
            NEW_FLOAT_EXT => { self.new_float() }
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
            NIL_EXT => { Ok(BertTerm::List(vec![])) }
            LIST_EXT => { self.list() }
            STRING_EXT => { self.string() }
            BINARY_EXT => { self.binary() }
            SMALL_BIG_EXT => {
                let len = self.eat_u8()?;
                self.bigint(len as usize)
            }
            LARGE_BIG_EXT => {
                let len = self.eat_u32_be()?;
                self.bigint(len as usize)
            }
            tag => { Err(BertError::InvalidTag(offset, tag)) }
        }
    }

    fn small_integer(&mut self) -> Result<BertTerm> {
        let b = self.eat_u8()?;
        Ok(BertTerm::Int(b as i32))
    }

    fn integer(&mut self) -> Result<BertTerm> {
        let n = self.eat_i32_be()?;
        Ok(BertTerm::Int(n))
    }

    fn old_float(&mut self) -> Result<BertTerm> {
        let offset = self.pos;
        let mut s = String::new();
        while !self.eof() && self.peek()? != 0 {
            s.push(self.eat_char()?);
        }

        while !self.eof() && self.peek()? == 0 {
            let _ = self.eat_u8()?;
        }

        s.parse::<f64>()
            .map_err(|_| BertError::InvalidFloat(offset))
            .map(|f| BertTerm::Float(f))
    }

    fn new_float(&mut self) -> Result<BertTerm> {
        let raw_bytes = self.eat_u64_be()?;
        let f: f64 = unsafe { mem::transmute(raw_bytes) };
        Ok(BertTerm::Float(f))
    }

    fn atom(&mut self, len: usize) -> Result<BertTerm> {
        let offset = self.pos;
        let mut bytes: Vec<u8> = Vec::with_capacity(len);
        for _ in 0 .. len {
            bytes.push(self.eat_u8()?);
        }
        ISO_8859_1.decode(&bytes, DecoderTrap::Strict)
            .map(|s| BertTerm::Atom(s))
            .map_err(|_| BertError::InvalidLatin1Atom(offset))
    }

    fn atom_utf8(&mut self, len: usize) -> Result<BertTerm> {
        let offset = self.pos;
        let mut buf = Vec::with_capacity(len);
        for _ in 0 .. len {
            buf.push(self.eat_u8()?);
        }
        String::from_utf8(buf)
            .map(|s| BertTerm::Atom(s))
            .map_err(|_| BertError::InvalidUTF8Atom(offset))
    }

    fn tuple(&mut self, len: usize) -> Result<BertTerm> {
        let mut terms = Vec::with_capacity(len);
        for _ in 0 .. len {
            terms.push(self.bert_term()?);
        }
        Ok(BertTerm::Tuple(terms))
    }

    fn string(&mut self) -> Result<BertTerm> {
        let len = self.eat_u16_be()?;
        let mut bytes = Vec::with_capacity(len as usize);
        for _ in 0 .. len {
            bytes.push(self.eat_u8()?);
        }
        Ok(BertTerm::String(bytes))
    }

    fn binary(&mut self) -> Result<BertTerm> {
        let len = self.eat_u32_be()?;
        let mut bytes = Vec::with_capacity(len as usize);
        for _ in 0 .. len {
            bytes.push(self.eat_u8()?);
        }
        Ok(BertTerm::Binary(bytes))
    }

    fn list(&mut self) -> Result<BertTerm> {
        let len = self.eat_u32_be()?;
        let mut terms = Vec::with_capacity(len as usize + 1);
        for _ in 0 .. len {
            terms.push(self.bert_term()?);
        }
        let tail = self.bert_term()?;
        match tail {
            BertTerm::List(_) => (),
            last_term => { terms.push(last_term); }
        };
        Ok(BertTerm::List(terms))
    }

    fn bigint(&mut self, len: usize) -> Result<BertTerm> {
        let sign = self.eat_u8()?;
        let mut sum: bigint::BigInt = Zero::zero();
        let mut pos: bigint::BigInt = One::one();
        for _ in 0 .. len {
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

    // Low-level parsing methods
    fn eof(&self) -> bool {
        self.pos >= self.contents.len()
    }

    fn peek(&self) -> Result<u8> {
        if self.eof() {
            return Err(BertError::EOF(self.pos));
        } else {
            return Ok(self.contents[self.pos]);
        }
    }

    fn eat_u8(&mut self) -> Result<u8> {
        let b = self.peek()?;
        self.pos += 1;
        return Ok(b);
    }

    fn eat_char(&mut self) -> Result<char> {
        let b = self.eat_u8()?;
        return Ok(b as char);
    }

    fn eat_u16_be(&mut self) -> Result<u16> {
        let b0 = self.eat_u8()? as u16;
        let b1 = self.eat_u8()? as u16;
        return Ok((b0 << 8) + b1)
    }

    fn eat_i32_be(&mut self) -> Result<i32> {
        let b0 = self.eat_u8()? as i32;
        let b1 = self.eat_u8()? as i32;
        let b2 = self.eat_u8()? as i32;
        let b3 = self.eat_u8()? as i32;
        return Ok((b0 << 24) + (b1 << 16) + (b2 << 8) + b3)
    }

    fn eat_u32_be(&mut self) -> Result<u32> {
        let b0 = self.eat_u8()? as u32;
        let b1 = self.eat_u8()? as u32;
        let b2 = self.eat_u8()? as u32;
        let b3 = self.eat_u8()? as u32;
        return Ok((b0 << 24) + (b1 << 16) + (b2 << 8) + b3)
    }

    fn eat_u64_be(&mut self) -> Result<u64> {
        let mut n: u64 = 0;
        n += (self.eat_u8()? as u64) << 56;
        n += (self.eat_u8()? as u64) << 48;
        n += (self.eat_u8()? as u64) << 40;
        n += (self.eat_u8()? as u64) << 32;
        n += (self.eat_u8()? as u64) << 24;
        n += (self.eat_u8()? as u64) << 16;
        n += (self.eat_u8()? as u64) << 8;
        n += self.eat_u8()? as u64;
        return Ok(n);
    }
}
