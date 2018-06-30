extern crate itoa;

use std::fmt::{self, Write as FmtWrite};
use std::io::{self, Write as IoWrite};
use std::iter;

use num::bigint;

use consts::*;

pub const DEFAULT_INDENT_WIDTH: usize = 2;
pub const DEFAULT_MAX_TERMS_PER_LINE: usize = 4;


#[derive(Debug, PartialEq)]
pub enum BertTerm {
    Nil,
    Int(i32),
    BigInt(bigint::BigInt),
    Float(f64),
    Atom(String),
    Tuple(Vec<BertTerm>),
    List(Vec<BertTerm>),
    Map(Vec<BertTerm>, Vec<BertTerm>),
    String(Vec<u8>),
    Binary(Vec<u8>)
}

impl BertTerm {
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

    fn is_proplist(&self) -> bool {
        match *self {
            BertTerm::List(ref elems) =>
                elems.iter().all(|e| e.is_proplist_entry()),
            _ => false
        }
    }

    fn is_proplist_entry(&self) -> bool {
        match *self {
            BertTerm::Tuple(ref elems) => is_proplist_tuple(elems),
            _ => false
        }
    }
}

fn is_proplist_tuple(elems: &[BertTerm]) -> bool {
    match elems {
        [BertTerm::Atom(_), _] => true,
        [BertTerm::String(_), _] => true,
        [BertTerm::Binary(_), _] => true,
        _ => false
    }
}


/// Outputs a vector of BertTerms to stdout.
pub fn pp_bert(terms: Vec<BertTerm>, indent_width: usize, terms_per_line: usize) {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for t in terms {
        let pp = PrettyPrinter::new(&t, indent_width, terms_per_line);
        let _ = writeln!(stdout, "{}", pp);
    }
}

/// Outputs a BertTerm as JSON to stdout.
pub fn pp_json(terms: Vec<BertTerm>, _: usize, _: usize) {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for t in terms {
        let pp = JsonPrettyPrinter { term: &t, transform_proplists: false };
        let _ = writeln!(stdout, "{}", pp);
    }
}


/// Outputs a BertTerm as JSON to stdout;
/// Erlang proplists are converted to JSON objects.
pub fn pp_json_proplist(terms: Vec<BertTerm>, _: usize, _: usize) {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for t in terms {
        let pp = JsonPrettyPrinter { term: &t, transform_proplists: true };
        let _ = writeln!(stdout, "{}", pp);
    }
}


pub struct PrettyPrinter<'a> {
    term: &'a BertTerm,
    indent_width: usize,
    max_terms_per_line: usize
}

impl <'a> fmt::Display for PrettyPrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write_term(self.term, f, 0)
    }
}


impl <'a> PrettyPrinter<'a> {
    /// Creates a pretty printer for `term` where sub-terms
    /// are indented with a width of `indent_width` and a
    /// maximum of `max_terms_per_line` basic terms (i.e.,
    /// integers, floats, strings) can be printed per line.
    pub fn new(term: &'a BertTerm,
               indent_width: usize,
               max_terms_per_line: usize) -> Self {
        PrettyPrinter { term, indent_width, max_terms_per_line }
    }


    fn write_term(&self, term: &BertTerm, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        match *term {
            BertTerm::Nil => f.write_str("[]"),
            BertTerm::Int(n) => itoa::fmt(f, n),
            BertTerm::BigInt(ref n) => write!(f, "{}", n),
            BertTerm::Float(x) => write!(f, "{}", x),
            BertTerm::Atom(ref s) => f.write_str(s),
            BertTerm::String(ref bytes) => self.write_string(bytes, f, "\"", "\""),
            BertTerm::Binary(ref bytes) => self.write_string(bytes, f, "<<\"", "\">>"),
            BertTerm::List(ref terms) => self.write_collection(terms, f, depth, '[', ']'),
            BertTerm::Tuple(ref terms) => self.write_collection(terms, f, depth, '{', '}'),
            BertTerm::Map(ref keys, ref vals) => self.write_map(keys, vals, f, depth)
        }
    }


    fn write_string(&self,
                    bytes: &[u8],
                    f: &mut fmt::Formatter,
                    open: &str,
                    close: &str) -> fmt::Result {
        // It's faster to write all the characters in a String
        // and then write a str than it is to write each character
        // individually.
        let mut buf = String::with_capacity(4096);
        buf.push_str(open);
        for &b in bytes {
            if is_printable(b) {
                buf.push(b as char);
            } else {
                buf.push_str(&format!("\\x{:02x}", b));
            }
        }
        buf.push_str(close);
        f.write_str(&buf)
    }


    fn write_collection(&self,
                        terms: &[BertTerm],
                        f: &mut fmt::Formatter,
                        depth: usize,
                        open: char,
                        close: char) -> fmt::Result {
        let multi_line = !self.is_small_collection(terms);

        // Every element will have the same indentation,
        // so pre-compute it once.
        let prefix =
            if multi_line {
                self.indentation(depth+1)
            } else {
                String::new()
            };

        f.write_char(open)?;
        let mut comma = "";
        for t in terms {
            f.write_str(comma)?;
            f.write_str(&prefix)?;
            self.write_term(t, f, depth + 1)?;
            comma = ", ";
        }

        if multi_line {
            f.write_str(&self.indentation(depth))?;
        }

        f.write_char(close)
    }


    fn write_map(&self,
                 keys: &[BertTerm],
                 vals: &[BertTerm],
                 f: &mut fmt::Formatter,
                 depth: usize) -> fmt::Result {
        let multi_line =
            !self.is_small_collection(keys) || !self.is_small_collection(vals);
        let prefix =
            if multi_line {
                self.indentation(depth+1)
            } else {
                String::new()
            };

        f.write_str("#{")?;
        let mut comma = "";
        for i in 0 .. keys.len() {
            f.write_str(comma)?;
            f.write_str(&prefix)?;
            self.write_term(&keys[i], f, depth + 1)?;
            f.write_str(" => ")?;
            self.write_term(&vals[i], f, depth + 1)?;
            comma = ", ";
        }

        if multi_line {
            f.write_str(&self.indentation(depth))?;
        }
        f.write_str("}")
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


pub struct JsonPrettyPrinter<'a> {
    term: &'a BertTerm,
    transform_proplists: bool
}


impl <'a> fmt::Display for JsonPrettyPrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.term.write_json(f, self.transform_proplists)
    }
}

impl BertTerm {
    fn write_json(&self, f: &mut fmt::Formatter, transform_proplists: bool) -> fmt::Result {
        use self::BertTerm::*;
        match *self {
            Nil => f.write_str("[]"),
            Int(n) => write!(f, "{}", n),
            BigInt(ref b) => write!(f, "\"{}\"", b),
            Float(x) => write!(f, "{}", x),
            Atom(ref s) => write!(f, "\"{}\"", s),
            List(ref terms) =>
                if transform_proplists && self.is_proplist() {
                    f.write_char('{')?;
                    let mut first = true;
                    for term in terms {
                        if !first { f.write_char(',')?; }
                        first = false;
                        term.write_as_kv_pair(f, transform_proplists)?;
                    }
                    f.write_char('}')
                } else {
                    self.write_list(f, terms, transform_proplists)
                }
            Tuple(ref terms) => self.write_list(f, terms, transform_proplists),
            Binary(ref bytes) | String(ref bytes) => {
                let mut buf = ::std::string::String::with_capacity(4096);
                buf.push('"');
                for b in bytes {
                    if must_be_escaped(*b) {
                        buf.push('\\');
                        buf.push(*b as char);
                    } else if is_printable(*b) {
                        buf.push(*b as char);
                    } else {
                        buf.push_str(&format!("\\u{:04x}", *b));
                    }
                }
                buf.push('"');
                f.write_str(&buf)
            }
            Map(ref keys, ref values) => {
                f.write_char('{')?;
                let mut first = true;
                for (key, value) in keys.iter().zip(values) {
                    if !first { f.write_char(',')?; }
                    first = false;
                    key.write_json(f, transform_proplists)?;
                    f.write_char(':')?;
                    value.write_json(f, transform_proplists)?;
                }
                f.write_char('}')
            }
        }
    }

    fn write_as_kv_pair(&self, f: &mut fmt::Formatter, transform_proplists: bool) -> fmt::Result {
        match *self {
            BertTerm::Tuple(ref kv) => {
                assert_eq!(2, kv.len());
                kv[0].write_json(f, transform_proplists)?;
                f.write_char(':')?;
                kv[1].write_json(f, transform_proplists)
            }
            _ => {
                let pp = PrettyPrinter::new(
                    self, DEFAULT_INDENT_WIDTH, DEFAULT_MAX_TERMS_PER_LINE);
                panic!("{} is not a proplist item", pp)
            }
        }
    }

    fn write_list(&self, f: &mut fmt::Formatter, terms: &[BertTerm], transform_proplists: bool) -> fmt::Result {
        f.write_char('[')?;
        let mut first = true;
        for term in terms {
            if !first { f.write_char(',')?; }
            first = false;
            term.write_json(f, transform_proplists)?;
        }
        f.write_char(']')
    }
}


fn is_printable(b: u8) -> bool {
    b >= 0x20 && b <= 0x7e
}

fn must_be_escaped(b: u8) -> bool {
    b == b'"' || b == b'\\'
}


impl BertTerm {
    pub fn write_bert(&self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(4096);
        out.push(BERT_MAGIC_NUMBER);
        self.dump_bert(&mut out);
        return out;
    }

    fn dump_bert(&self, out: &mut Vec<u8>) {
        match *self {
            BertTerm::Nil => out.push(NIL_EXT),
            BertTerm::Int(n) => {
                if n >= 0 && n < 256 {
                    out.push(SMALL_INTEGER_EXT);
                    out.push(n as u8);
                } else {
                    out.push(INTEGER_EXT);
                    out.push( ((n >> 24) & 0xff) as u8 );
                    out.push( ((n >> 16) & 0xff) as u8 );
                    out.push( ((n >> 8) & 0xff) as u8 );
                    out.push( (n & 0xff) as u8 );
                }
            }
            BertTerm::BigInt(ref b) => {
                let (sign, bytes) = b.to_bytes_le();
                let len = bytes.len();
                if len < 256 {
                    out.push(SMALL_BIG_EXT);
                    out.push(len as u8);
                } else {
                    out.push(LARGE_BIG_EXT);
                    out.push( ((len >> 24) & 0xff) as u8 );
                    out.push( ((len >> 16) & 0xff) as u8 );
                    out.push( ((len >> 8) & 0xff) as u8 );
                    out.push( (len & 0xff) as u8 );
                }
                if sign == bigint::Sign::Minus {
                    out.push(1);
                } else {
                    out.push(0);
                }
                out.extend(bytes);
            }
            BertTerm::Float(f) => {
                let n = f.to_bits();
                out.push(NEW_FLOAT_EXT);
                out.push( ((n >> 56) & 0xff) as u8 );
                out.push( ((n >> 48) & 0xff) as u8 );
                out.push( ((n >> 40) & 0xff) as u8 );
                out.push( ((n >> 32) & 0xff) as u8 );
                out.push( ((n >> 24) & 0xff) as u8 );
                out.push( ((n >> 16) & 0xff) as u8 );
                out.push( ((n >> 8) & 0xff) as u8 );
                out.push( (n & 0xff) as u8 );
            }
            BertTerm::Tuple(ref terms) => {
                let len = terms.len();
                if len < 256 {
                    out.push(SMALL_TUPLE_EXT);
                    out.push(len as u8);
                } else {
                    out.push(LARGE_TUPLE_EXT);
                    out.push( ((len >> 24) & 0xff) as u8 );
                    out.push( ((len >> 16) & 0xff) as u8 );
                    out.push( ((len >>  8) & 0xff) as u8 );
                    out.push( (len & 0xff) as u8 );
                }
                for t in terms {
                    t.dump_bert(out);
                }
            }
            BertTerm::List(ref terms) => {
                let len = terms.len();
                out.push(LIST_EXT);
                out.push( ((len >> 24) & 0xff) as u8 );
                out.push( ((len >> 16) & 0xff) as u8 );
                out.push( ((len >>  8) & 0xff) as u8 );
                out.push( (len & 0xff) as u8 );
                for t in terms {
                    t.dump_bert(out);
                }
                out.push(NIL_EXT);
            }
            BertTerm::Map(ref keys, ref vals) => {
                let len = keys.len();
                out.push(MAP_EXT);
                out.push( ((len >> 24) & 0xff) as u8 );
                out.push( ((len >> 16) & 0xff) as u8 );
                out.push( ((len >>  8) & 0xff) as u8 );
                out.push( (len & 0xff) as u8 );
                for (k, v) in keys.iter().zip(vals) {
                    k.dump_bert(out);
                    v.dump_bert(out);
                }
            }
            BertTerm::Atom(ref chars) => {
                let bytes = chars.bytes();
                let len = bytes.len();
                out.push(ATOM_UTF8_EXT);
                out.push( ((len >> 8) & 0xff) as u8 );
                out.push( (len & 0xff) as u8 );
                out.extend(bytes);
            }
            BertTerm::String(ref bytes) => {
                let len = bytes.len();
                out.push(STRING_EXT);
                out.push( ((len >> 8) & 0xff) as u8 );
                out.push( (len & 0xff) as u8 );
                out.extend(bytes);
            }
            BertTerm::Binary(ref bytes) => {
                let len = bytes.len();
                out.push(BINARY_EXT);
                out.push( ((len >> 24) & 0xff) as u8 );
                out.push( ((len >> 16) & 0xff) as u8 );
                out.push( ((len >>  8) & 0xff) as u8 );
                out.push( (len & 0xff) as u8 );
                out.extend(bytes);
            }
        }
    }
}
