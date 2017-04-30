extern crate ppbert;
#[macro_use] extern crate clap;

use std::io::{self, Read, Write};
use std::fs::File;
use std::process::exit;

use clap::{Arg, App};

use ppbert::parser;
use ppbert::bertterm::BertTerm;
use ppbert::error::Result;

fn main() {
    let matches = App::new("ppbert")
        .version(crate_version!())
        .author("Vincent Foley")
        .about("Pretty print structure encoded in Erlang's External Term Format")
        .arg(Arg::with_name("input_files")
             .value_name("BERT FILE")
             .multiple(true))
        .get_matches();

    let files: Vec<&str> = match matches.values_of("input_files") {
        Some(files) => files.collect(),
        None => vec!["-"]
    };

    let mut return_code = 0;
    for file in files {
        let _ = parse_and_print(file)
            .map(|ref t| println!("{}", t))
            .map_err(|ref e| {
                return_code = 1;
                writeln!(&mut io::stderr(), "ppbert: {}: {}", file, e)
            });
    }
    exit(return_code);
}


fn parse_and_print(file: &str) -> Result<BertTerm> {
    let mut buf: Vec<u8> = Vec::new();
    if file == "-" {
        let mut stdin = io::stdin();
        stdin.read_to_end(&mut buf)?;
    } else {
        let mut f = File::open(file)?;
        f.read_to_end(&mut buf)?;
    }
    let mut parser = parser::Parser::new(buf);
    return parser.parse();
}
