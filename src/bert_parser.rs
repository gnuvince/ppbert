extern crate nom;
extern crate num;

use std::mem;
use std::fmt;

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

#[derive(Debug, PartialEq)]
pub enum BertTerm {
    Int(i32),
    BigInt(bigint::BigInt),
    Float(f64),
    Atom(String),
    Tuple(Vec<BertTerm>),
    List(Vec<BertTerm>),
    Binary(Vec<u8>)
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
                let mut first = true;
                for t in ts {
                    if !first { write!(f, ", "); }
                    write!(f, "{}", t);
                    first = false;
                }
                write!(f, "{}", '}')
            }
            BertTerm::List(ref ts) => {
                write!(f, "[");
                let mut first = true;
                for t in ts {
                    if !first { write!(f, ", "); }
                    write!(f, "{}", t);
                    first = false;
                }
                write!(f, "]")
            }
            BertTerm::Binary(ref bytes) => {
                write!(f, "<<");
                let mut first = true;
                for b in bytes {
                    if !first { write!(f, ", "); }
                    write!(f, "{}", b);
                    first = false;
                }
                write!(f, ">>")
            }
        }
    }
}


#[derive(Debug)]
pub enum BertError {
    Unknown,
    InvalidMagicNumber
}

named!(bert_magic_number, tag!([BERT_MAGIC_NUMBER]));

fn small_integer(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([SMALL_INTEGER_EXT]));
    let (i, n) = try_parse!(i, nom::be_u8);
    IResult::Done(i, BertTerm::Int(n as i32))
}

#[test]
fn test_small_integer() {
    for i in 0 .. u8::max_value() {
        let buf = &[SMALL_INTEGER_EXT, i];
        let t = small_integer(buf);
        assert_eq!(t, IResult::Done(&b""[..], BertTerm::Int(i as i32)));
    }
}


fn integer(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([INTEGER_EXT]));
    let (i, n) = try_parse!(i, nom::be_i32);
    IResult::Done(i, BertTerm::Int(n))
}


#[test]
fn test_integer() {
    let buf0 = &[INTEGER_EXT, 0, 0, 0, 1];
    let buf1 = &[INTEGER_EXT, 0, 0, 1, 0];
    let buf2 = &[INTEGER_EXT, 0, 1, 0, 0];
    let buf3 = &[INTEGER_EXT, 1, 0, 0, 0];

    assert_eq!(integer(buf0), IResult::Done(&b""[..], BertTerm::Int(1)));
    assert_eq!(integer(buf1), IResult::Done(&b""[..], BertTerm::Int(1 << 8)));
    assert_eq!(integer(buf2), IResult::Done(&b""[..], BertTerm::Int(1 << 16)));
    assert_eq!(integer(buf3), IResult::Done(&b""[..], BertTerm::Int(1 << 24)));
}


fn atom(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([ATOM_EXT]));
    let (i, len) = try_parse!(i, nom::be_u16);
    let (i, atom_name) = try_parse!(i, take_str!(len));
    IResult::Done(i, BertTerm::Atom(atom_name.to_string()))
}

#[test]
fn test_atom() {
    let buf = &[ATOM_EXT, 0, 2, b'a', b'b'];
    assert_eq!(atom(buf), IResult::Done(&b""[..], BertTerm::Atom("ab".to_string())));
}

fn small_atom_utf8(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([SMALL_ATOM_UTF8_EXT]));
    let (i, len) = try_parse!(i, nom::be_u8);
    let (i, atom_name) = try_parse!(i, take_str!(len));
    IResult::Done(i, BertTerm::Atom(atom_name.to_string()))
}

fn atom_utf8(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([ATOM_UTF8_EXT]));
    let (i, len) = try_parse!(i, nom::be_u16);
    let (i, atom_name) = try_parse!(i, take_str!(len));
    IResult::Done(i, BertTerm::Atom(atom_name.to_string()))
}

fn small_tuple(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([SMALL_TUPLE_EXT]));
    let (i, arity) = try_parse!(i, nom::be_u8);
    let (i, tuple) = try_parse!(i, count!(bert_term, arity as usize));
    IResult::Done(i, BertTerm::Tuple(tuple))
}

#[test]
fn test_small_tuple() {
    let buf = &[SMALL_TUPLE_EXT, 2, ATOM_EXT, 0, 2, b'o', b'k', SMALL_INTEGER_EXT, 42];
    assert_eq!(small_tuple(buf),
               IResult::Done(&b""[..],
                             BertTerm::Tuple(vec![BertTerm::Atom("ok".to_string()),
                                                  BertTerm::Int(42)])));
}

fn large_tuple(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([LARGE_TUPLE_EXT]));
    let (i, arity) = try_parse!(i, nom::be_u32);
    let (i, tuple) = try_parse!(i, count!(bert_term, arity as usize));
    IResult::Done(i, BertTerm::Tuple(tuple))
}

fn nil(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([NIL_EXT]));
    IResult::Done(i, BertTerm::List(vec![]))
}

#[test]
fn test_nil() {
    let buf = &[NIL_EXT];
    assert_eq!(nil(buf), IResult::Done(&b""[..], BertTerm::List(vec![])));
}

fn string_like_list(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([STRING_EXT]));
    let (i, len) = try_parse!(i, nom::be_u16);
    let (i, nums) = try_parse!(i, take!(len as usize));
    let elements = nums.iter().map(|n| BertTerm::Int(*n as i32)).collect();
    IResult::Done(i, BertTerm::List(elements))
}

#[test]
fn string_like_list_test() {
    use self::BertTerm::*;
    let buf = &[STRING_EXT, 0, 5, b'h', b'e', b'l', b'l', b'o'];
    assert_eq!(string_like_list(buf),
               IResult::Done(&b""[..], List(vec![Int(104), Int(101), Int(108), Int(108), Int(111)])));
}

fn list(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([LIST_EXT]));
    let (i, len) = try_parse!(i, nom::be_u32);
    let (i, mut elements) = try_parse!(i, count!(bert_term, len as usize));
    let (i, tail) = try_parse!(i, bert_term);
    match tail {
        BertTerm::List(_) => (),
        last_term => { elements.push(last_term); }
    };
    IResult::Done(i, BertTerm::List(elements))
}

#[test]
fn test_list() {
    use self::BertTerm::*;
    let buf0 = &[
        LIST_EXT, 0, 0, 0, 2,
        SMALL_INTEGER_EXT, 42,
        INTEGER_EXT, 0, 1, 0, 0,
        NIL_EXT
    ];
    let buf1 = &[
        LIST_EXT, 0, 0, 0, 2,
        SMALL_INTEGER_EXT, 42,
        INTEGER_EXT, 0, 1, 0, 0,
        SMALL_INTEGER_EXT, 84
    ];
    assert_eq!(list(buf0), IResult::Done(&b""[..], List(vec![Int(42), Int(65536)])));
    assert_eq!(list(buf1), IResult::Done(&b""[..], List(vec![Int(42), Int(65536), Int(84)])));
}


fn binary(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([BINARY_EXT]));
    let (i, len) = try_parse!(i, nom::be_u32);
    let (i, elements) = try_parse!(i, count!(nom::be_u8, len as usize));
    IResult::Done(i, BertTerm::Binary(elements))
}

#[test]
fn binary_test() {
    let buf = &[BINARY_EXT, 0, 0, 0, 4, 1, 3, 3, 7];
    assert_eq!(binary(buf), IResult::Done(&b""[..], BertTerm::Binary(vec![1,3,3,7])));
}

fn is_zero(b: u8) -> bool { b == 0 }
fn is_non_zero(b: u8) -> bool { b != 0 }

fn old_float(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([FLOAT_EXT]));
    let (i, bytes) = try_parse!(i, take_while!(is_non_zero));
    let (i, _) = try_parse!(i, take_while!(is_zero));
    let mut s = String::new();
    for b in bytes { s.push(*b as char); }
    let f = s.parse::<f64>().unwrap();
    IResult::Done(i, BertTerm::Float(f))
}

fn new_float(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([NEW_FLOAT_EXT]));
    let (i, raw_bytes) = try_parse!(i, nom::be_u64);
    let f: f64 = unsafe { mem::transmute(raw_bytes) };
    IResult::Done(i, BertTerm::Float(f))
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

fn small_big_int(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([SMALL_BIG_EXT]));
    let (i, len) = try_parse!(i, nom::be_u8);
    let (i, sign) = try_parse!(i, nom::be_u8);
    let (i, digits) = try_parse!(i, take!(len as usize));
    let b = compute_big_int(sign == 1, &digits);
    IResult::Done(i, BertTerm::BigInt(b))
}

fn large_big_int(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, tag!([LARGE_BIG_EXT]));
    let (i, len) = try_parse!(i, nom::be_u32);
    let (i, sign) = try_parse!(i, nom::be_u8);
    let (i, digits) = try_parse!(i, take!(len as usize));
    let b = compute_big_int(sign == 1, &digits);
    IResult::Done(i, BertTerm::BigInt(b))
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
       )
);

pub fn parse(i: &[u8]) -> IResult<&[u8], BertTerm> {
    let (i, _) = try_parse!(i, bert_magic_number);
    let (i, t) = try_parse!(i, bert_term);
    IResult::Done(i, t)
}
