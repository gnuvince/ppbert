#[macro_use]
extern crate nom;

use std::io;
use std::io::Read;

use nom::{IResult, ErrorKind};


static BERT_MAGIC_NUMBER: u8 = 131;

static SMALL_INTEGER_EXT: u8 = 97;
static INTEGER_EXT: u8 = 98;
static ATOM_EXT: u8 = 100;

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
    Unknown,
    InvalidMagicNumber
}

named!(bert_magic_number, tag!([BERT_MAGIC_NUMBER]));

fn small_integer(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([SMALL_INTEGER_EXT]));
    let (i2, n) = try_parse!(i1, nom::be_u8);
    IResult::Done(i2, BertTerm::Int(n as i32))
}


fn integer(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([INTEGER_EXT]));
    let (i2, n) = try_parse!(i1, nom::be_i32);
    IResult::Done(i2, BertTerm::Int(n))
}

fn atom(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([ATOM_EXT]));
    let (i2, len) = try_parse!(i1, nom::be_u16);
    let (i3, atom_name) = try_parse!(i2, take_str!(len));
    IResult::Done(i3, BertTerm::Atom(atom_name.to_string()))
}

named!(parse_bert<&[u8], BertTerm>, chain!(
    bert_magic_number ~
    t: alt!(small_integer | integer | atom)
    ,
    || { t }
));

fn main() {
    let mut stdin = io::stdin();
    let mut buf: Vec<u8> = Vec::new();

    stdin.read_to_end(&mut buf);
    println!("{:?}", parse_bert(&buf[..]));
}
