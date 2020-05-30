use std::io;

use crate::prelude::*;
use crate::pp::PrettyPrinter;
use crate::pp::utils::*;

pub struct JsonPrettyPrinter {
    transform_proplists: bool
}

impl PrettyPrinter for JsonPrettyPrinter {
    fn write(&self, term: &BertTerm, mut w: Box<dyn io::Write>) -> Result<()> {
        self.write_term(term, &mut w)?;
        writeln!(w, "")?;
        return Ok(());
    }
}

impl JsonPrettyPrinter {
    pub fn new(transform_proplists: bool) -> Self {
        JsonPrettyPrinter { transform_proplists }
    }

    fn write_term<W: io::Write>(&self, term: &BertTerm, w: &mut W) -> io::Result<()> {
        match *term {
            BertTerm::Nil => w.write_all(b"[]"),
            BertTerm::Int(n) => itoa::write(w, n).map(|_| ()),
            BertTerm::BigInt(ref b) => write!(w, "\"{}\"", b),
            BertTerm::Float(x) => {
                let mut buf = ryu::Buffer::new();
                w.write_all(buf.format(x).as_bytes())
            }
            BertTerm::Atom(ref s) => {
                if s == "true" {
                    write!(w, "true")
                } else if s == "false" {
                    write!(w, "false")
                } else {
                    write!(w, "\"{}\"", s)
                }
            }
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
