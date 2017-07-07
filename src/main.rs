extern crate ppbert;
#[macro_use] extern crate clap;

use std::io::{self, Read, Write};
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
        .arg(Arg::with_name("skip_pretty_print")
             .help("Only parse the file(s)")
             .short("s")
             .long("--skip-pretty-print"))
        .arg(Arg::with_name("bert2")
             .help("Parse .bert2 files")
             .short("2")
             .long("bert2"))
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
    let skip_pretty_print = matches.is_present("skip_pretty_print");
    let bert2 = matches.is_present("bert2");

    let mut return_code = 0;
    for file in files {
        let res =
            if bert2 {
                handle_file(file, skip_pretty_print, verbose,
                            indent_level, max_per_line,
                            parse_bert2,
                            bertterm::pp_bert2)
            } else {
                handle_file(file, skip_pretty_print, verbose,
                            indent_level, max_per_line,
                            parse_bert1,
                            bertterm::pp_bert1)
            };
        match res {
            Ok(()) => (),
            Err(e) => {
                return_code = 1;
                let _ = writeln!(&mut io::stderr(), "ppbert: {}: {}", file, e);
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
        let _ = writeln!(&mut io::stderr(), "ppbert: parse time: {}.{:09}s", dur1.as_secs(), dur1.subsec_nanos());
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
        let _ = writeln!(&mut io::stderr(), "ppbert: print time: {}.{:09}s", dur2.as_secs(), dur2.subsec_nanos());
    }

    return Ok(());
}


fn parse_bert1(buf: Vec<u8>) -> Result<BertTerm> {
    let mut parser = parser::Parser::new(buf);
    return parser.parse();
}

fn parse_bert2(buf: Vec<u8>) -> Result<Vec<BertTerm>> {
    let mut parser = parser::Parser::new(buf);
    return parser.parse_bert2();
}
