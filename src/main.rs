#[macro_use]
extern crate nom;

use std::io;
use std::io::Read;

use nom::{IResult, ErrorKind};


static BERT_MAGIC_NUMBER: u8 = 131;

static SMALL_INTEGER_EXT: u8 = 97;
static INTEGER_EXT: u8 = 98;
static ATOM_EXT: u8 = 100;
static SMALL_TUPLE_EXT: u8 = 104;
static LARGE_TUPLE_EXT: u8 = 105;
static NIL_EXT: u8 = 106;
static STRING_EXT: u8 = 107;
static LIST_EXT: u8 = 108;
static BINARY_EXT: u8 = 109;

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

fn small_tuple(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([SMALL_TUPLE_EXT]));
    let (i2, arity) = try_parse!(i1, nom::be_u8);
    let (i3, tuple) = try_parse!(i2, count!(bert_term, arity as usize));
    IResult::Done(i3, BertTerm::Tuple(tuple))
}

fn large_tuple(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([LARGE_TUPLE_EXT]));
    let (i2, arity) = try_parse!(i1, nom::be_u32);
    let (i3, tuple) = try_parse!(i2, count!(bert_term, arity as usize));
    IResult::Done(i3, BertTerm::Tuple(tuple))
}

fn nil(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([NIL_EXT]));
    IResult::Done(i1, BertTerm::List(vec![]))
}

fn string_like_list(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([STRING_EXT]));
    let (i2, len) = try_parse!(i1, nom::be_u16);
    let (i3, nums) = try_parse!(i2, count!(nom::be_u8, len as usize));
    let elements = nums.into_iter().map(|n| BertTerm::Int(n as i32)).collect();
    IResult::Done(i3, BertTerm::List(elements))
}

fn list(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([LIST_EXT]));
    let (i2, len) = try_parse!(i1, nom::be_u32);
    let (i3, mut elements) = try_parse!(i2, count!(bert_term, len as usize));
    let (i4, tail) = try_parse!(i3, bert_term);
    match tail {
        BertTerm::List(_) => (),
        last_term => { elements.push(last_term); }
    };
    IResult::Done(i4, BertTerm::List(elements))
}

fn binary(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([BINARY_EXT]));
    let (i2, len) = try_parse!(i1, nom::be_u32);
    let (i3, elements) = try_parse!(i2, count!(nom::be_u8, len as usize));
    IResult::Done(i3, BertTerm::Binary(elements))
}

named!(bert_term<&[u8], BertTerm>,
       alt!(small_integer
            | integer
            | atom
            | small_tuple
            | large_tuple
            | nil
            | list
            | string_like_list
            | binary
));

named!(parse_bert<&[u8], BertTerm>, do_parse!(
    bert_magic_number >> t: bert_term >> (t)
));

fn main() {
    let mut stdin = io::stdin();
    let mut buf: Vec<u8> = Vec::new();

    let _ = stdin.read_to_end(&mut buf);
    println!("{:?}", parse_bert(&buf[..]));
}
