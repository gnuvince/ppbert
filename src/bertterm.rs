extern crate itoa;

use std::io;
use std::iter;

use byteorder::{BigEndian, WriteBytesExt};
use num_bigint::{BigInt, Sign};

use consts::*;
use error::Result;

/// A enum representing a BertTerm
#[derive(Debug, PartialEq)]
pub enum BertTerm {
    /// The empty list
    Nil,

    /// A signed 32-bit integer
    Int(i32),

    /// A signed arbitrary-size integer
    BigInt(BigInt),

    /// A double-precision floating point number
    Float(f64),

    /// An atom
    Atom(String),

    /// A latin-1-encoded string
    String(Vec<u8>),

    /// An array of bytes
    Binary(Vec<u8>),

    /// A container for a fixed number of elements
    Tuple(Vec<BertTerm>),

    /// A container for an arbitrary number of elements
    List(Vec<BertTerm>),

    /// A container for key-to-value pairs
    Map(Vec<BertTerm>, Vec<BertTerm>),

}

impl BertTerm {
    /// Lists, tuples, and maps are not basic terms;
    /// everything else is.
    fn is_basic(&self) -> bool {
        match *self {
            BertTerm::Int(_)
            | BertTerm::BigInt(_)
            | BertTerm::Float(_)
            | BertTerm::Atom(_)
            | BertTerm::String(_)
            | BertTerm::Binary(_)
            | BertTerm::Nil => true,
            BertTerm::List(_)
            | BertTerm::Tuple(_)
            | BertTerm::Map(_, _) => false
        }
    }

    /// A term is a proplist if it has this shape:
    /// [ {atom|string|binary, term}* ]
    fn is_proplist(&self) -> bool {
        fn is_proplist_tuple(elems: &[BertTerm]) -> bool {
            match elems {
                [BertTerm::Atom(_), _] => true,
                [BertTerm::String(_), _] => true,
                [BertTerm::Binary(_), _] => true,
                _ => false
            }
        }

        fn is_proplist_entry(t: &BertTerm) -> bool {
            match *t {
                BertTerm::Tuple(ref elems) => is_proplist_tuple(elems),
                _ => false
            }
        }

        match *self {
            BertTerm::List(ref elems) =>
                elems.iter().all(|e| is_proplist_entry(e)),
            _ => false
        }
    }

    /// Writes a `BertTerm` into `W` using Erlang syntax.
    /// The output is indented and printed over multiple lines.
    ///
    /// - `indent_width`:
    ///     how many spaces to use for indentation
    /// - `max_terms_per_line`:
    ///     a list or a tuple made of basic terms, it may be be
    ///     printed on a single line of the number of elements does
    ///     not exceed this limit.
    pub fn write_as_erlang<W: io::Write>
        (&self, w: &mut W, indent_width: usize, max_terms_per_line: usize) -> Result<()>
    {
        let () = ErlangPrettyPrinter::new(&self, indent_width, max_terms_per_line).write(w)?;
        Ok(())
    }

    /// Writes a `BertTerm` into `W` using JSON syntax.
    /// The output is not indented and is printed on a single line.
    ///
    /// - `transform_prolists`:
    ///     Erlang proplists are sometimes used in place of maps.
    ///     To make the output of `write_as_json` easier to manipulate
    ///     in a tool like `jq`, the parameter `transform_prolists` is
    ///     given the value `true` and the proplists will be output as
    ///     JSON objects.
    pub fn write_as_json<W: io::Write>
        (&self, w: &mut W, transform_prolists: bool) -> Result<()>
    {
        let () = JsonPrettyPrinter::new(&self, transform_prolists).write(w)?;
        Ok(())
    }

    /// Writes a `BertTerm` into `W` encoded using Erlang's [External Term Format].
    ///
    /// [External Term Format]: http://erlang.org/doc/apps/erts/erl_ext_dist.html
    pub fn write_as_bert<W: io::Write>(&self, w: &mut W) -> Result<()> {
        BertWriter::new(&self).write(w)?;
        return Ok(());
    }
}

struct ErlangPrettyPrinter<'a> {
    term: &'a BertTerm,
    indent_width: usize,
    max_terms_per_line: usize
}

impl <'a> ErlangPrettyPrinter<'a> {
    /// Creates a pretty printer for `term` where sub-terms
    /// are indented with a width of `indent_width` and a
    /// maximum of `max_terms_per_line` basic terms (i.e.,
    /// integers, floats, strings) can be printed per line.
    fn new(term: &'a BertTerm, indent_width: usize, max_terms_per_line: usize) -> Self {
        ErlangPrettyPrinter { term, indent_width, max_terms_per_line }
    }

    fn write<W: io::Write>(&self, w: &mut W) -> Result<()> {
        self.write_term(&self.term, w, 0).map_err(|e| e.into())
    }

    fn write_term<W: io::Write>(&self, term: &BertTerm, w: &mut W, depth: usize) -> io::Result<()> {
        match *term {
            BertTerm::Nil => w.write_all(b"[]"),
            BertTerm::Int(n) => itoa::write(w, n).map(|_| ()),
            BertTerm::BigInt(ref n) => write!(w, "{}", n),
            BertTerm::Float(x) => write!(w, "{}", x),
            BertTerm::Atom(ref s) => w.write_all(s.as_bytes()),
            BertTerm::String(ref bytes) => self.write_string(bytes, w, b"\"", b"\""),
            BertTerm::Binary(ref bytes) => self.write_string(bytes, w, b"<<\"", b"\">>"),
            BertTerm::List(ref terms) => self.write_collection(terms, w, depth, b"[", b"]"),
            BertTerm::Tuple(ref terms) => self.write_collection(terms, w, depth, b"{", b"}"),
            BertTerm::Map(ref keys, ref vals) => self.write_map(keys, vals, w, depth)
        }
    }


    fn write_string<W: io::Write>
        (&self, bytes: &[u8], w: &mut W, open: &[u8], close: &[u8]) -> io::Result<()>
    {
        let mut start = 0;
        w.write_all(open)?;

        for (i, &b) in bytes.iter().enumerate() {
            if !is_printable(b) {
                w.write_all(&bytes[start .. i])?;
                start = i + 1;
                write!(w, "\\x{:02x}", b)?;
            }
        }

        // Write remaining bytes
        w.write_all(&bytes[start..])?;
        w.write_all(close)
    }


    fn write_collection<W: io::Write>
        (&self, terms: &[BertTerm], w: &mut W, depth: usize, open: &[u8], close: &[u8])
         -> io::Result<()>
    {
        let multi_line = !self.is_small_collection(terms);

        // Every element will have the same indentation,
        // so pre-compute it once.
        let prefix =
            if multi_line {
                self.indentation(depth+1)
            } else {
                String::new()
            };

        w.write_all(open)?;
        let mut comma = "";
        for t in terms {
            w.write_all(comma.as_bytes())?;
            w.write_all(prefix.as_bytes())?;
            self.write_term(t, w, depth + 1)?;
            comma = ", ";
        }

        if multi_line {
            w.write_all(&self.indentation(depth).as_bytes())?;
        }

        w.write_all(close)
    }


    fn write_map<W: io::Write>
        (&self, keys: &[BertTerm], vals: &[BertTerm], w: &mut W, depth: usize) -> io::Result<()>
    {
        let multi_line =
            !self.is_small_collection(keys) || !self.is_small_collection(vals);
        let prefix =
            if multi_line {
                self.indentation(depth+1)
            } else {
                String::new()
            };

        w.write_all(b"#{")?;
        let mut comma = "";
        for i in 0 .. keys.len() {
            w.write_all(comma.as_bytes())?;
            w.write_all(prefix.as_bytes())?;
            self.write_term(&keys[i], w, depth + 1)?;
            w.write_all(b" => ")?;
            self.write_term(&vals[i], w, depth + 1)?;
            comma = ", ";
        }

        if multi_line {
            w.write_all(&self.indentation(depth).as_bytes())?;
        }
        w.write_all(b"}")
    }

    fn is_small_collection(&self, terms: &[BertTerm]) -> bool {
        terms.len() <= self.max_terms_per_line &&
            terms.iter().all(BertTerm::is_basic)
    }

    fn indentation(&self, depth: usize) -> String {
        let nl = iter::once('\n');
        let spaces = iter::repeat(' ').take(depth * self.indent_width);
        nl.chain(spaces).collect()
    }
}


struct JsonPrettyPrinter<'a> {
    term: &'a BertTerm,
    transform_proplists: bool
}

impl <'a> JsonPrettyPrinter<'a> {
    fn new(term: &'a BertTerm, transform_proplists: bool) -> Self {
        JsonPrettyPrinter { term, transform_proplists }
    }

    fn write<W: io::Write>(&self, w: &mut W) -> Result<()> {
        self.write_term(&self.term, w).map_err(|e| e.into())
    }

    fn write_term<W: io::Write>(&self, term: &BertTerm, w: &mut W) -> io::Result<()> {
        match *term {
            BertTerm::Nil => w.write_all(b"[]"),
            BertTerm::Int(n) => itoa::write(w, n).map(|_| ()),
            BertTerm::BigInt(ref b) => write!(w, "\"{}\"", b),
            BertTerm::Float(x) => write!(w, "{}", x),
            BertTerm::Atom(ref s) => write!(w, "\"{}\"", s),
            BertTerm::List(ref terms) =>
                if self.transform_proplists && term.is_proplist() {
                    w.write_all(b"{")?;
                    let mut comma = "";
                    for term in terms {
                        w.write_all(comma.as_bytes())?;
                        comma = ",";
                        self.write_as_kv_pair(term, w)?;
                    }
                    w.write_all(b"}")
                } else {
                    self.write_list(terms, w)
                }
            BertTerm::Tuple(ref terms) => self.write_list(terms, w),
            BertTerm::Binary(ref bytes) | BertTerm::String(ref bytes) => {
                w.write_all(b"\"")?;
                let mut start = 0;
                for (i, &b) in bytes.iter().enumerate() {
                    if must_be_escaped(b) {
                        w.write_all(&bytes[start .. i])?;
                        start = i + 1;
                        write!(w, "\\{}", b as char)?;
                    } else if !is_printable(b) {
                        w.write_all(&bytes[start .. i])?;
                        start = i + 1;
                        write!(w, "\\u{:04x}", b)?;
                    }
                }
                w.write_all(&bytes[start..])?;
                w.write_all(b"\"")
            }
            BertTerm::Map(ref keys, ref values) => {
                w.write_all(b"{")?;
                let mut comma = "";
                for (key, value) in keys.iter().zip(values) {
                    w.write_all(comma.as_bytes())?;
                    comma = ",";
                    self.write_term(key, w)?;
                    w.write_all(b":")?;
                    self.write_term(value, w)?;
                }
                w.write_all(b"}")
            }
        }
    }

    fn write_as_kv_pair<W: io::Write>(&self, term: &BertTerm, w: &mut W) -> io::Result<()> {
        match *term {
            BertTerm::Tuple(ref kv) => {
                assert_eq!(2, kv.len());
                self.write_term(&kv[0], w)?;
                w.write_all(b":")?;
                self.write_term(&kv[1], w)
            }
            _ => {
                panic!("{:?} is not a proplist item", term)
            }
        }
    }

    fn write_list<W: io::Write>(&self, terms: &[BertTerm], w: &mut W) -> io::Result<()> {
        w.write_all(b"[")?;
        let mut comma = "";
        for term in terms {
            w.write_all(comma.as_bytes())?;
            comma = ",";
            self.write_term(term, w)?;
        }
        w.write_all(b"]")
    }
}


fn is_printable(b: u8) -> bool {
    b >= 0x20 && b <= 0x7e
}

fn must_be_escaped(b: u8) -> bool {
    b == b'"' || b == b'\\'
}

struct BertWriter<'a> {
    term: &'a BertTerm
}

impl <'a> BertWriter<'a> {
    fn new(term: &'a BertTerm) -> Self {
        BertWriter { term }
    }

    fn write<W: io::Write>(&self, w: &mut W) -> Result<()> {
        w.write_u8(BERT_MAGIC_NUMBER)?;
        self.write_bert(&self.term, w)?;
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
