#[macro_use]
extern crate nom;
extern crate num;

mod bert_parser;

use std::io;
use std::io::Read;

use nom::IResult;

fn main() {
    let mut stdin = io::stdin();
    let mut buf: Vec<u8> = Vec::new();

    let _ = stdin.read_to_end(&mut buf);
    match bert_parser::parse(&buf[..]) {
        IResult::Done(_, _) => { println!("ok"); }
        IResult::Error(e) => { println!("error: {:?}", e); }
        IResult::Incomplete(_) => { println!("incomplete"); }
    }
}
