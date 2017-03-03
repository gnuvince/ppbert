extern crate nom;
extern crate num;

use std::mem;
use nom::{IResult, ErrorKind};
use num::bigint;
use num::bigint::ToBigInt;
use num::traits::{Zero, One};

static BERT_MAGIC_NUMBER: u8 = 131;
static SMALL_INTEGER_EXT: u8 = 97;
static INTEGER_EXT: u8 = 98;
static FLOAT_EXT: u8 = 99;
static ATOM_EXT: u8 = 100;
static SMALL_TUPLE_EXT: u8 = 104;
static LARGE_TUPLE_EXT: u8 = 105;
static NIL_EXT: u8 = 106;
static STRING_EXT: u8 = 107;
static LIST_EXT: u8 = 108;
static BINARY_EXT: u8 = 109;
static SMALL_BIG_EXT: u8 = 110;
static LARGE_BIG_EXT: u8 = 111;
static ATOM_UTF8_EXT: u8 = 118;
static SMALL_ATOM_UTF8_EXT: u8 = 119;
static NEW_FLOAT_EXT: u8 = 70;

#[derive(Debug)]
pub enum BertTerm {
    Int(i32),
    BigInt(bigint::BigInt),
    Float(f64),
    Atom(String),
    Tuple(Vec<BertTerm>),
    List(Vec<BertTerm>),
    Binary(Vec<u8>)
}

#[derive(Debug)]
pub enum BertError {
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

fn small_atom_utf8(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([SMALL_ATOM_UTF8_EXT]));
    let (i2, len) = try_parse!(i1, nom::be_u8);
    let (i3, atom_name) = try_parse!(i2, take_str!(len));
    IResult::Done(i3, BertTerm::Atom(atom_name.to_string()))
}

fn atom_utf8(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([ATOM_UTF8_EXT]));
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

fn is_zero(b: u8) -> bool { b == 0 }
fn is_non_zero(b: u8) -> bool { b != 0 }

fn old_float(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([FLOAT_EXT]));
    let (i2, bytes) = try_parse!(i1, take_while!(is_non_zero));
    let (i3, _) = try_parse!(i2, take_while!(is_zero));
    let mut s = String::new();
    for b in bytes { s.push(*b as char); }
    let f = s.parse::<f64>().unwrap();
    IResult::Done(i3, BertTerm::Float(f))
}

fn new_float(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([NEW_FLOAT_EXT]));
    let (i2, raw_bytes) = try_parse!(i1, nom::be_u64);
    let f: f64 = unsafe { mem::transmute(raw_bytes) };
    IResult::Done(i2, BertTerm::Float(f))
}

fn compute_big_int(is_negative: bool, digits: &[u8]) -> bigint::BigInt {
    let mut sum: bigint::BigInt = Zero::zero();
    let mut pos: bigint::BigInt = One::one();
    for d in digits {
        let t = &pos * &(d.to_bigint().unwrap());
        sum = sum + &t;
        pos = pos * (256).to_bigint().unwrap();
    }
    if is_negative { sum = -sum; }
    return sum;
}

fn small_big_int(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([SMALL_BIG_EXT]));
    let (i2, len) = try_parse!(i1, nom::be_u8);
    let (i3, sign) = try_parse!(i2, nom::be_u8);
    let (i4, digits) = try_parse!(i3, count!(nom::be_u8, len as usize));
    let b = compute_big_int(sign == 1, &digits);
    IResult::Done(i4, BertTerm::BigInt(b))
}

fn large_big_int(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, tag!([LARGE_BIG_EXT]));
    let (i2, len) = try_parse!(i1, nom::be_u32);
    let (i3, sign) = try_parse!(i2, nom::be_u8);
    let (i4, digits) = try_parse!(i3, count!(nom::be_u8, len as usize));
    let b = compute_big_int(sign == 1, &digits);
    IResult::Done(i4, BertTerm::BigInt(b))
}


named!(bert_term<&[u8], BertTerm>,
       alt!(small_integer
            | integer
            | atom
            | small_atom_utf8
            | atom_utf8
            | small_tuple
            | large_tuple
            | nil
            | list
            | string_like_list
            | binary
            | old_float
            | new_float
            | small_big_int
            | large_big_int
));

pub fn parse(i0: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i1, _) = try_parse!(i0, bert_magic_number);
    let (i2, t) = try_parse!(i1, bert_term);
    IResult::Done(i2, t)
}
