use ppbert::consts::*;
use ppbert::error::{BertError, Result};

use num_bigint::{BigInt, ToBigInt};
use num_traits::{One, Zero};

use byteorder::{BigEndian, ReadBytesExt};

use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};

#[derive(Debug, Default)]
pub struct Terms {
    pub tags: Vec<Tag>,
    pub bytes: Vec<u8>,
    curr: usize,
}

impl Terms {
    pub fn clear(&mut self) {
        self.curr = 0;
        self.tags.clear();
        self.bytes.clear();
    }

    fn push_tag(&mut self, t: Tag) {
        self.tags.push(t);
        self.curr += 1;
    }

    fn pop_tag(&mut self) {
        self.tags.pop();
        self.curr -= 1;
    }

    fn push_bytes(&mut self, buf: &[u8]) -> (u32, u32) {
        let off = self.bytes.len();
        self.bytes.extend(buf);
        return (off as u32, buf.len() as u32);
    }
}

#[derive(Debug)]
pub enum Tag {
    Int(i32),
    // NB(vfoley): I use a box to reduce the size of `Term` from 40 bytes to 16 bytes.
    BigInt(Box<BigInt>),
    Float(f64),
    Atom { off: u32, len: u32 },
    String { off: u32, len: u32 },
    Binary { off: u32, len: u32 },
    Tuple(u32),
    List(u32),
    Proplist(u32),
    Map(u32),
}

impl Tag {
    fn is_proplist_key(&self) -> bool {
        matches!(
            *self,
            Tag::Atom { .. } | Tag::String { .. } | Tag::Binary { .. }
        )
    }
}

pub type ParserNext = fn(&mut BertParser, &mut Terms) -> Option<Result<()>>;
type ParserFn = for<'a, 'b> fn(&'a mut BertParser, &'b mut Terms) -> Result<()>;

pub struct BertParser {
    contents: Vec<u8>,
    pos: usize,
    parse_fns: [ParserFn; 256],
}

impl BertParser {
    pub fn new(contents: Vec<u8>) -> BertParser {
        let mut parse_fns = [Self::invalid_op_code as ParserFn; 256];
        parse_fns[SMALL_INTEGER_EXT as usize] = Self::small_integer;
        parse_fns[INTEGER_EXT as usize] = Self::integer;
        parse_fns[FLOAT_EXT as usize] = Self::old_float;
        parse_fns[NEW_FLOAT_EXT as usize] = Self::new_float;
        parse_fns[ATOM_EXT as usize] = Self::atom;
        parse_fns[ATOM_UTF8_EXT as usize] = Self::atom_utf8;
        parse_fns[SMALL_ATOM_EXT as usize] = Self::small_atom;
        parse_fns[SMALL_ATOM_UTF8_EXT as usize] = Self::small_atom_utf8;
        parse_fns[SMALL_TUPLE_EXT as usize] = Self::small_tuple;
        parse_fns[LARGE_TUPLE_EXT as usize] = Self::large_tuple;
        parse_fns[NIL_EXT as usize] = Self::nil;
        parse_fns[LIST_EXT as usize] = Self::list;
        parse_fns[STRING_EXT as usize] = Self::string;
        parse_fns[BINARY_EXT as usize] = Self::binary;
        parse_fns[SMALL_BIG_EXT as usize] = Self::small_big;
        parse_fns[LARGE_BIG_EXT as usize] = Self::large_big;
        parse_fns[MAP_EXT as usize] = Self::map;
        BertParser {
            contents,
            pos: 0,
            parse_fns,
        }
    }

    // "Iterators"
    pub fn bert1_next(&mut self, terms: &mut Terms) -> Option<Result<()>> {
        if self.eof() {
            return None;
        }
        if let Err(e) = self.magic_number() {
            return Some(Err(e));
        }
        if let Err(e) = self.bert_term(terms) {
            return Some(Err(e));
        }
        return Some(Ok(()));
    }

    pub fn bert2_next(&mut self, terms: &mut Terms) -> Option<Result<()>> {
        if self.eof() {
            return None;
        }
        if let Err(e) = self.parse_varint() {
            return Some(Err(e));
        }
        if let Err(e) = self.magic_number() {
            return Some(Err(e));
        }
        if let Err(e) = self.bert_term(terms) {
            return Some(Err(e));
        }
        return Some(Ok(()));
    }

    pub fn disk_log_next(&mut self, terms: &mut Terms) -> Option<Result<()>> {
        if self.eof() {
            return None;
        }
        if let Err(e) = self.disk_log_magic() {
            return Some(Err(e));
        }
        if let Err(e) = self.disk_log_opened_status() {
            return Some(Err(e));
        }
        if let Err(e) = self.disk_log_term(terms) {
            return Some(Err(e));
        }
        return Some(Ok(()));
    }

    // Parsers
    fn magic_number(&mut self) -> Result<()> {
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

    pub fn disk_log_term(&mut self, terms: &mut Terms) -> Result<()> {
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

        return self.bert_term(terms);
    }

    fn invalid_op_code(&mut self, _: &mut Terms) -> Result<()> {
        let opcode = self.contents[self.pos - 1];
        return Err(BertError::InvalidTag(self.pos, opcode));
    }

    fn bert_term(&mut self, terms: &mut Terms) -> Result<()> {
        let bytecode = self.eat_u8()?;
        let f = self.parse_fns[bytecode as usize];
        f(self, terms)?;
        return Ok(());
    }

    fn small_integer(&mut self, terms: &mut Terms) -> Result<()> {
        let b = self.eat_u8()?;
        terms.push_tag(Tag::Int(b as i32));
        return Ok(());
    }

    fn integer(&mut self, terms: &mut Terms) -> Result<()> {
        let n = self.eat_i32_be()?;
        terms.push_tag(Tag::Int(n));
        return Ok(());
    }

    fn small_big(&mut self, terms: &mut Terms) -> Result<()> {
        let len = self.eat_u8()? as u32;
        self.bigint(len, terms)
    }

    fn large_big(&mut self, terms: &mut Terms) -> Result<()> {
        let len = self.eat_u32_be()? as u32;
        self.bigint(len, terms)
    }

    fn bigint(&mut self, len: u32, terms: &mut Terms) -> Result<()> {
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
        terms.push_tag(Tag::BigInt(Box::new(sum)));
        return Ok(());
    }

    fn old_float(&mut self, terms: &mut Terms) -> Result<()> {
        let initial_pos = self.pos;
        let mut s = String::new();
        while !self.eof() && self.peek()? != 0 {
            s.push(self.eat_char()?);
        }

        while !self.eof() && self.peek()? == 0 {
            let _ = self.eat_u8()?;
        }

        let f = s
            .parse::<f64>()
            .map_err(|_| BertError::InvalidFloat(initial_pos))?;
        terms.push_tag(Tag::Float(f));
        return Ok(());
    }

    fn new_float(&mut self, terms: &mut Terms) -> Result<()> {
        let raw_bytes = self.eat_u64_be()?;
        let f: f64 = f64::from_bits(raw_bytes);
        terms.push_tag(Tag::Float(f));
        return Ok(());
    }

    fn atom(&mut self, terms: &mut Terms) -> Result<()> {
        let len = self.eat_u16_be()? as usize;
        self.atom_with_length(len, terms)
    }

    fn small_atom(&mut self, terms: &mut Terms) -> Result<()> {
        let len = self.eat_u8()? as usize;
        self.atom_with_length(len, terms)
    }

    fn atom_with_length(&mut self, len: usize, terms: &mut Terms) -> Result<()> {
        let initial_pos = self.pos;
        let bytes: &[u8] = self.eat_slice(len)?;
        let is_ascii = bytes.iter().all(|byte| *byte < 128);

        // Optimization: ASCII atoms represent the overwhelming
        // majority of use cases of atoms. When we read the bytes
        // of the atom, we record whether they are all ASCII
        // (i.e., small than 128); if it's the case, we don't
        // need to bother with latin-1 decoding.
        if is_ascii {
            let (off, len) = terms.push_bytes(bytes);
            terms.push_tag(Tag::Atom { off, len });
        } else {
            let mut atom_buf = String::with_capacity(len);
            ISO_8859_1
                .decode_to(&bytes, DecoderTrap::Strict, &mut atom_buf)
                .map_err(|_| BertError::InvalidLatin1Atom(initial_pos))?;
            let (off, len) = terms.push_bytes(atom_buf.as_bytes());
            terms.push_tag(Tag::Atom { off, len });
        }
        return Ok(());
    }

    fn atom_utf8(&mut self, terms: &mut Terms) -> Result<()> {
        let len = self.eat_u16_be()? as usize;
        self.atom_utf8_with_length(len, terms)
    }

    fn small_atom_utf8(&mut self, terms: &mut Terms) -> Result<()> {
        let len = self.eat_u8()? as usize;
        self.atom_utf8_with_length(len, terms)
    }

    fn atom_utf8_with_length(&mut self, len: usize, terms: &mut Terms) -> Result<()> {
        let initial_pos = self.pos;
        let buf: &[u8] = self.eat_slice(len)?;

        // Verify that the slice of bytes is a valid UTF-8 string.
        if let Err(_) = std::str::from_utf8(buf) {
            return Err(BertError::InvalidUTF8Atom(initial_pos));
        }

        let (off, len) = terms.push_bytes(buf);
        terms.push_tag(Tag::Atom { off, len });
        return Ok(());
    }

    fn small_tuple(&mut self, terms: &mut Terms) -> Result<()> {
        let len = self.eat_u8()? as u32;
        self.tuple(len, terms)
    }

    fn large_tuple(&mut self, terms: &mut Terms) -> Result<()> {
        let len = self.eat_u32_be()?;
        self.tuple(len, terms)
    }

    fn tuple(&mut self, len: u32, terms: &mut Terms) -> Result<()> {
        terms.push_tag(Tag::Tuple(len));
        for _ in 0..len {
            self.bert_term(terms)?;
        }
        return Ok(());
    }

    fn nil(&mut self, terms: &mut Terms) -> Result<()> {
        terms.push_tag(Tag::List(0));
        return Ok(());
    }

    fn list(&mut self, terms: &mut Terms) -> Result<()> {
        let mut is_proplist = true;
        let len = self.eat_u32_be()?;
        let list_index = terms.curr;
        // Add one in case of improper list, e.g., [1, 2, 3 | 4].
        terms.push_tag(Tag::List(len + 1));
        for _ in 0..len {
            let new_item_index = terms.curr;
            self.bert_term(terms)?;
            is_proplist = is_proplist
                && matches!(terms.tags[new_item_index], Tag::Tuple(2))
                && terms.tags[new_item_index + 1].is_proplist_key();
        }
        self.bert_term(terms)?;
        match terms.tags[terms.curr - 1] {
            Tag::List(0) => {
                terms.pop_tag();
                if is_proplist {
                    terms.tags[list_index] = Tag::Proplist(len);
                } else {
                    terms.tags[list_index] = Tag::List(len);
                }
            }
            _ => (),
        };
        return Ok(());
    }

    // TODO(vfoley): ensure no duplicate keys
    fn map(&mut self, terms: &mut Terms) -> Result<()> {
        let len = self.eat_u32_be()?;
        terms.push_tag(Tag::Map(len));
        for _ in 0..len {
            self.bert_term(terms)?; // Key
            self.bert_term(terms)?; // Value
        }
        return Ok(());
    }

    fn string(&mut self, terms: &mut Terms) -> Result<()> {
        let len = self.eat_u16_be()? as usize;
        let bytes = self.eat_slice(len)?;
        let (off, len) = terms.push_bytes(bytes);
        terms.push_tag(Tag::String { off, len });
        return Ok(());
    }

    fn binary(&mut self, terms: &mut Terms) -> Result<()> {
        let len = self.eat_u32_be()? as usize;
        let bytes = self.eat_slice(len)?;
        let (off, len) = terms.push_bytes(bytes);
        terms.push_tag(Tag::Binary { off, len });
        return Ok(());
    }

    // Low-level parsing methods
    fn eof(&self) -> bool {
        self.pos >= self.contents.len()
    }

    fn peek(&self) -> Result<u8> {
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

    fn can_read(&self, n: usize) -> bool {
        (self.pos + n) <= self.contents.len()
    }

    fn eat_slice(&mut self, len: usize) -> Result<&[u8]> {
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
        let mut bytes = self.eat_slice(2)?;
        let n = bytes.read_u16::<BigEndian>()?;
        return Ok(n);
    }

    fn eat_i32_be(&mut self) -> Result<i32> {
        let mut bytes = self.eat_slice(4)?;
        let n = bytes.read_i32::<BigEndian>()?;
        return Ok(n);
    }

    fn eat_u32_be(&mut self) -> Result<u32> {
        let mut bytes = self.eat_slice(4)?;
        let n = bytes.read_u32::<BigEndian>()?;
        return Ok(n);
    }

    fn eat_u64_be(&mut self) -> Result<u64> {
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
