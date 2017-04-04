extern crate nom;
extern crate ppbert;

use std::io;
use std::io::Read;

use nom::IResult;

use ppbert::bert_parser;
use ppbert::pretty;

fn main() {
    let mut stdin = io::stdin();
    let mut buf: Vec<u8> = Vec::new();

    let _ = stdin.read_to_end(&mut buf);
    match bert_parser::parse(&buf[..]) {
        IResult::Done(rest, ref term) if rest.is_empty() => {
            //println!("{}", term);
            pretty::print(&term, 0);
        }
        IResult::Done(rest, _) => { println!("input not all consumed, remaining: {:?}", rest); }
        IResult::Error(e) => { println!("error: {:?}", e); }
        IResult::Incomplete(_) => { println!("incomplete"); }
    }
}
