use std::fmt;

use num::bigint;

const INDENT_WIDTH: usize = 2;
const MAX_TERMS_PER_LINE: usize = 4;

#[derive(Debug, PartialEq)]
pub enum BertTerm {
    Int(i32),
    BigInt(bigint::BigInt),
    Float(f64),
    Atom(String),
    Tuple(Vec<BertTerm>),
    List(Vec<BertTerm>),
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
            | BertTerm::Binary(_) => true,
            BertTerm::List(_)
            | BertTerm::Tuple(_) => false
        }
    }
}


impl fmt::Display for BertTerm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_term(self, f, 0)
    }
}

fn write_term(term: &BertTerm, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match *term {
        BertTerm::Int(n) => write!(f, "{}", n),
        BertTerm::BigInt(ref n) => write!(f, "{}", n),
        BertTerm::Float(x) => write!(f, "{}", x),
        BertTerm::Atom(ref s) => write!(f, "{}", s),
        BertTerm::String(ref bytes) => write_string(f, bytes, "\"", "\""),
        BertTerm::Binary(ref bytes) => write_string(f, bytes, "<<\"", "\">>"),
        BertTerm::List(ref terms) => write_collection(f, terms, indent, '[', ']'),
        BertTerm::Tuple(ref terms) => write_collection(f, terms, indent, '{', '}')
    }
}

fn write_string(f: &mut fmt::Formatter,
                bytes: &[u8],
                open: &str,
                close: &str) -> fmt::Result {
    write!(f, "{}", open)?;
    for &b in bytes {
        if is_printable(b) {
            write!(f, "{}", b as char)?;
        } else {
            write!(f, "\\x{:02}", b)?;
        }
    }
    write!(f, "{}", close)
}

fn write_collection(f: &mut fmt::Formatter,
                    terms: &[BertTerm],
                    indent: usize,
                    open: char,
                    close: char) -> fmt::Result {
    let multi_line = !is_small_collection(terms);
    write!(f, "{}", open)?;

    let mut first = true;
    for t in terms {
        if !first { write!(f, ", ")?; }
        if multi_line {
            write!(f, "\n")?;
            write_indentation(f, indent + 1)?;
        }
        write_term(t, f, indent + 1)?;
        first = false;
    }

    if multi_line {
        write!(f, "\n")?;
        write_indentation(f, indent)?;
    }

    write!(f, "{}", close)
}

fn is_small_collection(terms: &[BertTerm]) -> bool {
    terms.len() <= MAX_TERMS_PER_LINE && terms.iter().all(BertTerm::is_basic)
}

fn is_printable(b: u8) -> bool {
    b >= 0x20 && b <= 0x7e
}

fn write_indentation(f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
    for _ in 0 .. depth * INDENT_WIDTH {
        write!(f, " ")?;
    }
    Ok(())
}
