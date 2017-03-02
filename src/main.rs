#[macro_use]
extern crate nom;

use std::io;
use std::io::Read;

static BERT_MAGIC_NUMBER: u8 = 131;

#[derive(Debug)]
enum BertTerm {
    Int(i32),
    Float(f64),
    Atom(String),
    Tuple(Vec<BertTerm>),
    List(Vec<BertTerm>),
    Binary(Vec<u8>)
}

#[derive(Debug)]
enum BertError {
    InvalidMagicNumber
}

fn main() {
}
