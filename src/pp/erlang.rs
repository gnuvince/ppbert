use std::io;

use crate::pp::utils::*;
use crate::pp::PrettyPrinter;
use crate::prelude::*;

const SPACES: [u8; 4096] = [b' '; 4096];

pub struct ErlangPrettyPrinter {
    indent_width: usize,
    max_terms_per_line: usize,
    terminator: &'static str,
}

impl PrettyPrinter for ErlangPrettyPrinter {
    fn write(&self, term: &BertTerm, mut w: Box<dyn io::Write>) -> Result<()> {
        self.write_term(term, &mut w, 0)?;
        writeln!(w, "{}", self.terminator)?;
        return Ok(());
    }
}

impl ErlangPrettyPrinter {
    /// Creates a pretty printer for `term` where sub-terms
    /// are indented with a width of `indent_width` and a
    /// maximum of `max_terms_per_line` basic terms (i.e.,
    /// integers, floats, strings) can be printed per line.
    pub fn new(indent_width: usize, max_terms_per_line: usize, terminator: &'static str) -> Self {
        ErlangPrettyPrinter {
            indent_width,
            max_terms_per_line,
            terminator,
        }
    }

    fn write_term<W: io::Write>(&self, term: &BertTerm, w: &mut W, depth: usize) -> io::Result<()> {
        match *term {
            BertTerm::Nil => w.write_all(b"[]"),
            BertTerm::Int(n) => itoa::write(w, n).map(|_| ()),
            BertTerm::BigInt(ref n) => write!(w, "{}", n),
            BertTerm::Float(x) => {
                let mut buf = ryu::Buffer::new();
                w.write_all(buf.format(x).as_bytes())
            }
            BertTerm::Atom(ref s) => w.write_all(s.as_bytes()),
            BertTerm::String(ref bytes) => self.write_string(bytes, w, b"\"", b"\""),
            BertTerm::Binary(ref bytes) => self.write_string(bytes, w, b"<<\"", b"\">>"),
            BertTerm::List(ref terms) => self.write_collection(terms, w, depth, b"[", b"]"),
            BertTerm::Tuple(ref terms) => self.write_collection(terms, w, depth, b"{", b"}"),
            BertTerm::Map(ref keys, ref vals) => self.write_map(keys, vals, w, depth),
        }
    }

    fn write_string<W: io::Write>(
        &self,
        bytes: &[u8],
        w: &mut W,
        open: &[u8],
        close: &[u8],
    ) -> io::Result<()> {
        let mut start = 0;
        w.write_all(open)?;

        for (i, &b) in bytes.iter().enumerate() {
            if must_be_escaped(b) {
                w.write_all(&bytes[start..i])?;
                start = i + 1;
                write!(w, "\\{}", b as char)?;
            } else if !is_printable(b) {
                w.write_all(&bytes[start..i])?;
                start = i + 1;
                write!(w, "\\x{:02x}", b)?;
            }
        }

        // Write remaining bytes
        w.write_all(&bytes[start..])?;
        w.write_all(close)
    }

    fn write_collection<W: io::Write>(
        &self,
        terms: &[BertTerm],
        w: &mut W,
        depth: usize,
        open: &[u8],
        close: &[u8],
    ) -> io::Result<()> {
        let multi_line = !self.is_small_collection(terms);

        // Every element will have the same indentation,
        // so pre-compute it once.
        let prefix = if multi_line {
            self.indentation(depth + 1)
        } else {
            &[]
        };

        w.write_all(open)?;
        let mut comma: &[u8] = b"";
        for t in terms {
            w.write_all(comma)?;
            w.write_all(prefix)?;
            self.write_term(t, w, depth + 1)?;
            comma = b", ";
        }

        if multi_line {
            w.write_all(&self.indentation(depth))?;
        }

        w.write_all(close)
    }

    fn write_map<W: io::Write>(
        &self,
        keys: &[BertTerm],
        vals: &[BertTerm],
        w: &mut W,
        depth: usize,
    ) -> io::Result<()> {
        let multi_line = !self.is_small_collection(keys) || !self.is_small_collection(vals);
        let prefix = if multi_line {
            self.indentation(depth + 1)
        } else {
            &[]
        };

        w.write_all(b"#{")?;
        let mut comma: &[u8] = b"";
        for i in 0..keys.len() {
            w.write_all(comma)?;
            w.write_all(prefix)?;
            self.write_term(&keys[i], w, depth + 1)?;
            w.write_all(b" => ")?;
            self.write_term(&vals[i], w, depth + 1)?;
            comma = b", ";
        }

        if multi_line {
            w.write_all(&self.indentation(depth))?;
        }
        w.write_all(b"}")
    }

    fn is_small_collection(&self, terms: &[BertTerm]) -> bool {
        terms.len() <= self.max_terms_per_line && terms.iter().all(BertTerm::is_basic)
    }

    fn indentation(&self, depth: usize) -> &[u8] {
        let n = usize::min(SPACES.len(), depth * self.indent_width);
        return &SPACES[..n];
    }
}
