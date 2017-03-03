#[macro_use]
extern crate nom;

mod bert_parser;

use std::io;
use std::io::Read;


fn main() {
    let mut stdin = io::stdin();
    let mut buf: Vec<u8> = Vec::new();

    let _ = stdin.read_to_end(&mut buf);
    println!("{:?}", bert_parser::parse(&buf[..]));
}
