use std::fmt;

use num::bigint;

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

fn fmt_separated_by<T: fmt::Display>(f: &mut fmt::Formatter, terms: &[T], sep: &str) {
    let mut first = true;
    for t in terms {
        if !first { let _ = write!(f, "{}", sep); }
        let _ = write!(f, "{}", t);
        first = false;
    }
}


impl fmt::Display for BertTerm {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BertTerm::Int(n) => { write!(f, "{}", n) }
            BertTerm::BigInt(ref n) => { write!(f, "{}", n) }
            BertTerm::Float(x) => { write!(f, "{}", x) }
            BertTerm::Atom(ref s) => { write!(f, "{}", s) }
            BertTerm::Tuple(ref ts) => {
                write!(f, "{}", '{');
                fmt_separated_by(f, ts, ", ");
                write!(f, "{}", '}')
            }
            BertTerm::List(ref ts) => {
                write!(f, "[");
                fmt_separated_by(f, ts, ", ");
                write!(f, "]")
            }
            BertTerm::String(ref bytes) => {
                write!(f, "\"");
                for &b in bytes {
                    if b >= 0x20 && b <= 0x7e {
                        write!(f, "{}", b as char);
                    } else {
                        write!(f, "\\x{:02x}", b);
                    }
                }
                write!(f, "\"")
            }
            BertTerm::Binary(ref bytes) => {
                write!(f, "<<");
                fmt_separated_by(f, bytes, ", ");
                write!(f, ">>")
            }
        }
    }
}
