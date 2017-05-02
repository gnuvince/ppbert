use std::fmt;

use num::bigint;


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
}

impl fmt::Display for BertTerm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pp = PrettyPrinter::new(self,
                                    DEFAULT_INDENT_WIDTH,
                                    DEFAULT_MAX_TERMS_PER_LINE);
        write!(f, "{}", pp)
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


    fn write_term(&self, term: &BertTerm, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
        match *term {
            BertTerm::Nil => write!(f, "[]"),
            BertTerm::Int(n) => write!(f, "{}", n),
            BertTerm::BigInt(ref n) => write!(f, "{}", n),
            BertTerm::Float(x) => write!(f, "{}", x),
            BertTerm::Atom(ref s) => write!(f, "{}", s),
            BertTerm::String(ref bytes) => self.write_string(bytes, f, "\"", "\""),
            BertTerm::Binary(ref bytes) => self.write_string(bytes, f, "<<\"", "\">>"),
            BertTerm::List(ref terms) => self.write_collection(terms, f, indent, '[', ']'),
            BertTerm::Tuple(ref terms) => self.write_collection(terms, f, indent, '{', '}'),
            BertTerm::Map(ref keys, ref vals) => self.write_map(keys, vals, f, indent)
        }
    }


    fn write_string(&self,
                    bytes: &[u8],
                    f: &mut fmt::Formatter,
                    open: &str,
                    close: &str) -> fmt::Result {
        write!(f, "{}", open)?;
        for &b in bytes {
            if is_printable(b) {
                write!(f, "{}", b as char)?;
            } else {
                write!(f, "\\x{:02x}", b)?;
            }
        }
        write!(f, "{}", close)
    }


    fn write_collection(&self,
                        terms: &[BertTerm],
                        f: &mut fmt::Formatter,
                        indent: usize,
                        open: char,
                        close: char) -> fmt::Result {
        let multi_line = !self.is_small_collection(terms);
        write!(f, "{}", open)?;

        let mut first = true;
        for t in terms {
            if !first { write!(f, ", ")?; }
            if multi_line {
                write!(f, "\n")?;
                self.indent(f, indent + 1)?;
            }
            self.write_term(t, f, indent + 1)?;
            first = false;
        }

        if multi_line {
            write!(f, "\n")?;
            self.indent(f, indent)?;
        }

        write!(f, "{}", close)
    }


    fn write_map(&self,
                 keys: &[BertTerm],
                 vals: &[BertTerm],
                 f: &mut fmt::Formatter,
                 indent: usize) -> fmt::Result {
        let mult_line =
            !self.is_small_collection(keys) || !self.is_small_collection(vals);
        write!(f, "#{{")?;

        for i in 0 .. keys.len() {
            if i > 0 { write!(f, ", ")?; }
            if mult_line {
                write!(f, "\n")?;
                self.indent(f, indent + 1)?;
            }
            self.write_term(&keys[i], f, indent + 1)?;
            write!(f, " => ")?;
            self.write_term(&vals[i], f, indent + 1)?;
        }

        if mult_line {
            write!(f, "\n")?;
            self.indent(f, indent)?;
        }
        write!(f, "}}")
    }


    fn indent(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        for _ in 0 .. depth * self.indent_width {
            write!(f, " ")?;
        }
        Ok(())
    }


    fn is_small_collection(&self, terms: &[BertTerm]) -> bool {
        terms.len() <= self.max_terms_per_line &&
            terms.iter().all(BertTerm::is_basic)
    }
}



fn is_printable(b: u8) -> bool {
    b >= 0x20 && b <= 0x7e
}
