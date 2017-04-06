extern crate ppbert;

use std::io;
use std::io::Read;

use ppbert::parser;
use ppbert::pretty;

fn main() {
    let mut stdin = io::stdin();
    let mut buf: Vec<u8> = Vec::new();

    let _ = stdin.read_to_end(&mut buf);
    let mut parser = parser::Parser::new(buf);
    match parser.parse() {
        Ok(ref term) => { pretty::print(term, 0); }
        Err(error) => { println!("ppbert: {}", error); }
    }
}
