extern crate ppbert;
#[macro_use] extern crate clap;

use std::io::{self, Read};
use std::fs::File;
use std::process::exit;
use std::time::Instant;

use clap::{Arg, App};

use ppbert::bertterm::{
    self,
    BertTerm,
    DEFAULT_INDENT_WIDTH,
    DEFAULT_MAX_TERMS_PER_LINE
};
use ppbert::error::Result;
use ppbert::parser;

fn main() {
    let matches = App::new("ppbert")
        .version(crate_version!())
        .author("Vincent Foley")
        .about("Pretty print structure encoded in Erlang's External Term Format")
        .arg(Arg::with_name("input_files")
             .value_name("FILES")
             .multiple(true))
        .arg(Arg::with_name("indent_width")
             .help("Number of spaces to indent nested terms")
             .value_name("num")
             .short("i")
             .long("--indent-width")
             .takes_value(true))
        .arg(Arg::with_name("max_per_line")
             .help("Number of spaces to indent nested terms")
             .value_name("num")
             .short("m")
             .long("--max-terms-per-line")
             .takes_value(true))
        .arg(Arg::with_name("verbose")
             .help("Enable verbose mode")
             .short("v")
             .long("--verbose"))
        .arg(Arg::with_name("parse")
             .help("Parse the input, don't pretty print it")
             .short("p")
             .long("--parse"))
        .arg(Arg::with_name("bert2")
             .help("Parse .bert2 files")
             .short("2")
             .long("bert2"))
        .arg(Arg::with_name("json")
             .help("Output in JSON")
             .short("j")
             .long("json"))
        .get_matches();

    let files: Vec<&str> = match matches.values_of("input_files") {
        Some(files) => files.collect(),
        None => vec!["-"]
    };

    let indent_level = value_t!(matches, "indent_width", usize)
        .unwrap_or(DEFAULT_INDENT_WIDTH);
    let max_per_line = value_t!(matches, "max_per_line", usize)
        .unwrap_or(DEFAULT_MAX_TERMS_PER_LINE);
    let verbose = matches.is_present("verbose");
    let parse_only = matches.is_present("parse");

    let parse_fn =
        if matches.is_present("bert2") {parse_bert2} else {parse_bert1};
    let output_fn =
        if matches.is_present("json") {bertterm::pp_json} else {bertterm::pp_bert};

    let mut return_code = 0;
    for file in files {
        let res = handle_file(file, parse_only, verbose,
                              indent_level, max_per_line,
                              parse_fn, output_fn);
        match res {
            Ok(()) => (),
            Err(e) => {
                return_code = 1;
                eprintln!("ppbert: {}: {}", file, e);
            }
        }
    }
    exit(return_code);
}


fn handle_file<T>(
    file: &str,
    parse_only: bool,
    verbose: bool,
    indent: usize,
    terms_per_line: usize,
    parse_fn: fn(Vec<u8>) -> Result<T>,
    pp_fn: fn(T, usize, usize) -> ()
) -> Result<()> {

    // Read file or stdin into buffer
    let mut buf: Vec<u8> = Vec::new();
    if file == "-" {
        let mut stdin = io::stdin();
        stdin.read_to_end(&mut buf)?;
    } else {
        let mut f = File::open(file)?;
        f.read_to_end(&mut buf)?;
    }

    // Parse input
    let now = Instant::now();
    let parse_output = parse_fn(buf)?;
    let dur1 = now.elapsed();

    if verbose {
        eprintln!("ppbert: parse time: {}.{:09}s", dur1.as_secs(), dur1.subsec_nanos());
    }

    // Early exit if parse-only
    if parse_only {
        return Ok(());
    }

    // Pretty print
    let now = Instant::now();
    pp_fn(parse_output, indent, terms_per_line);
    let dur2 = now.elapsed();

    if verbose {
        eprintln!("ppbert: print time: {}.{:09}s", dur2.as_secs(), dur2.subsec_nanos());
    }

    return Ok(());
}


fn parse_bert1(buf: Vec<u8>) -> Result<Vec<BertTerm>> {
    let mut parser = parser::Parser::new(buf);
    let term = parser.parse()?;
    return Ok(vec![term]);
}

fn parse_bert2(buf: Vec<u8>) -> Result<Vec<BertTerm>> {
    let mut parser = parser::Parser::new(buf);
    return parser.parse_bert2();
}
