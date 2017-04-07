extern crate ppbert;

use std::io;
use std::io::{Read, Write};

use ppbert::parser;
use ppbert::pretty;

fn main() {
    let mut stdin = io::stdin();
    let mut buf: Vec<u8> = Vec::new();

    let _ = stdin.read_to_end(&mut buf);
    let mut parser = parser::Parser::new(buf);
    match parser.parse() {
        Ok(ref term) => {
            pretty::print(term, 0);
            println!();
        }
        Err(error) => {
            let _ = writeln!(&mut io::stderr(), "ppbert: {}", error);
        }
    }
}
