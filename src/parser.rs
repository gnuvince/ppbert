extern crate num;

use std::mem;

use num::bigint;
use num::bigint::ToBigInt;
use num::traits::{Zero, One};

use bertterm::BertTerm;
use error::{Result, BertError};

static BERT_MAGIC_NUMBER: u8 = 131;
static SMALL_INTEGER_EXT: u8 = 97;
static INTEGER_EXT: u8 = 98;
static FLOAT_EXT: u8 = 99;
static ATOM_EXT: u8 = 100;
static SMALL_ATOM_EXT: u8 = 115;
static SMALL_TUPLE_EXT: u8 = 104;
static LARGE_TUPLE_EXT: u8 = 105;
static NIL_EXT: u8 = 106;
static STRING_EXT: u8 = 107;
static LIST_EXT: u8 = 108;
static BINARY_EXT: u8 = 109;
static SMALL_BIG_EXT: u8 = 110;
static LARGE_BIG_EXT: u8 = 111;
static ATOM_UTF8_EXT: u8 = 118;
static SMALL_ATOM_UTF8_EXT: u8 = 119;
static NEW_FLOAT_EXT: u8 = 70;

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
        return self.bert_term();
    }

    // Parsers
    fn magic_number(&mut self) -> Result<()> {
        let magic = self.eat_u8()?;
        if magic != BERT_MAGIC_NUMBER {
            return Err(BertError::InvalidMagicNumber);
        } else {
            return Ok(());
        }
    }

    fn bert_term(&mut self) -> Result<BertTerm> {
        let tag = self.eat_u8()?;
        if tag == SMALL_INTEGER_EXT {
            self.small_integer()
        } else if tag == INTEGER_EXT {
            self.integer()
        } else if tag == FLOAT_EXT {
            self.old_float()
        } else if tag == NEW_FLOAT_EXT {
            self.new_float()
        } else if tag == ATOM_EXT {
            let len = self.eat_u16_be()? as usize;
            self.atom(len)
        } else if tag == SMALL_ATOM_EXT {
            let len = self.eat_u8()? as usize;
            self.atom(len)
        } else if tag == ATOM_UTF8_EXT {
            let len = self.eat_u16_be()? as usize;
            self.atom_utf8(len)
        } else if tag == SMALL_ATOM_UTF8_EXT {
            let len = self.eat_u8()? as usize;
            self.atom_utf8(len)
        } else if tag == SMALL_TUPLE_EXT {
            let len = self.eat_u8()? as usize;
            self.tuple(len)
        } else if tag == LARGE_TUPLE_EXT {
            let len = self.eat_u32_be()? as usize;
            self.tuple(len)
        } else if tag == NIL_EXT {
            Ok(BertTerm::List(vec![]))
        } else if tag == LIST_EXT {
            self.list()
        } else if tag == STRING_EXT {
            self.string()
        } else if tag == BINARY_EXT {
            self.binary()
        } else if tag == SMALL_BIG_EXT {
            let len = self.eat_u8()?;
            self.bigint(len as usize)
        } else if tag == LARGE_BIG_EXT {
            let len = self.eat_u32_be()?;
            self.bigint(len as usize)
        } else {
            Err(BertError::InvalidTag(tag))
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
        let mut s = String::new();
        while self.peek()? != 0 {
            s.push(self.eat_char()?);
        }

        while self.peek()? == 0 {
            let _ = self.eat_u8()?;
        }

        s.parse::<f64>()
            .map_err(|_| BertError::InvalidFloat)
            .map(|f| BertTerm::Float(f))
    }

    fn new_float(&mut self) -> Result<BertTerm> {
        let raw_bytes = self.eat_u64_be()?;
        let f: f64 = unsafe { mem::transmute(raw_bytes) };
        Ok(BertTerm::Float(f))
    }

    fn atom(&mut self, len: usize) -> Result<BertTerm> {
        let mut s = String::with_capacity(len);
        for _ in 0 .. len {
            s.push(self.eat_char()?);
        }
        Ok(BertTerm::Atom(s))
    }

    fn atom_utf8(&mut self, len: usize) -> Result<BertTerm> {
        let mut buf = Vec::with_capacity(len);
        for _ in 0 .. len {
            buf.push(self.eat_u8()?);
        }
        String::from_utf8(buf)
            .map(|s| BertTerm::Atom(s))
            .map_err(|_| BertError::InvalidUTF8Atom)
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
        self.pos > self.contents.len()
    }

    fn peek(&self) -> Result<u8> {
        self.peek_at(self.pos)
    }

    fn peek_at(&self, offset: usize) -> Result<u8> {
        if self.eof() {
            return Err(BertError::EOF);
        } else {
            return Ok(self.contents[offset]);
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
