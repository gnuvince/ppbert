extern crate ppbert;
#[macro_use] extern crate clap;

use std::io::{self, BufReader, Read};
use std::fs::File;
use std::process::exit;
use std::time::Instant;

use clap::{Arg, App};

use ppbert::bertterm::{self, BertTerm};
use ppbert::error::Result;
use ppbert::parser;


const DEFAULT_INDENT_WIDTH: &str = "2";
const DEFAULT_MAX_TERMS_PER_LINE: &str = "5";
const PROG_NAME: &str = "ppbert";

fn main() {
    let matches = App::new(PROG_NAME)
        .version(crate_version!())
        .author("Vincent Foley")
        .about("Pretty print structure encoded in Erlang's External Term Format")
        .arg(Arg::with_name("input_files")
             .value_name("FILES")
             .multiple(true))
        .arg(Arg::with_name("indent_width")
             .help("Indents with <num> spaces")
             .value_name("num")
             .short("i")
             .long("indent-width")
             .default_value(DEFAULT_INDENT_WIDTH)
             .takes_value(true))
        .arg(Arg::with_name("max_per_line")
             .help("Prints at most <num> basic terms per line")
             .value_name("num")
             .short("m")
             .long("max-terms-per-line")
             .default_value(DEFAULT_MAX_TERMS_PER_LINE)
             .takes_value(true))
        .arg(Arg::with_name("verbose")
             .help("Enables verbose mode")
             .short("v")
             .long("verbose"))
        .arg(Arg::with_name("parse")
             .help("Parses the input, doesn't pretty print it")
             .short("p")
             .long("parse"))
        .arg(Arg::with_name("bert2")
             .help("Parses .bert2 files")
             .conflicts_with("disk_log")
             .short("2")
             .long("bert2"))
        .arg(Arg::with_name("disk_log")
             .help("Parses disk_log files")
             .conflicts_with("bert2")
             .short("d")
             .long("disk-log"))
        .arg(Arg::with_name("json")
             .help("Outputs in JSON")
             .short("j")
             .long("json"))
        .arg(Arg::with_name("transform-proplists")
             .help("Transforms proplists into JSON objects (only valid with --json)")
             .long("transform-proplists"))
        .get_matches();

    let files: Vec<&str> = match matches.values_of("input_files") {
        Some(files) => files.collect(),
        None => vec!["-"]
    };
    let indent_level = value_t!(matches, "indent_width", usize).unwrap();
    let max_per_line = value_t!(matches, "max_per_line", usize).unwrap();
    let verbose = matches.is_present("verbose");
    let parse_only = matches.is_present("parse");
    let transform_proplists = matches.is_present("transform-proplists");

    let parse_fn =
        if matches.is_present("bert2") {
            parse_bert2
        } else if matches.is_present("disk_log") {
            parse_disk_log
        } else {
            parse_bert1
        };
    let output_fn =
        if matches.is_present("json") {
            if transform_proplists {
                bertterm::pp_json_proplist
            } else {
                bertterm::pp_json
            }
        } else {
            if transform_proplists {
                eprintln!("{}: warning: --transform-proplists is only valid with the --json flag",
                          PROG_NAME);
            }
            bertterm::pp_bert
        };

    let mut return_code = 0;
    for file in files {
        let res = handle_file(file, parse_only, verbose,
                              indent_level, max_per_line,
                              parse_fn, output_fn);
        match res {
            Ok(()) => (),
            Err(e) => {
                return_code = 1;
                eprintln!("{}: {}: {}", PROG_NAME, file, e);
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
    let now = Instant::now();
    if file == "-" {
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        stdin.read_to_end(&mut buf)?;
    } else {
        let f = File::open(file)?;
        let mut rdr = BufReader::new(f);
        rdr.read_to_end(&mut buf)?;
    }
    let dur = now.elapsed();
    if verbose {
        eprintln!("{}: read time: {}.{:09}",
                  PROG_NAME, dur.as_secs(), dur.subsec_nanos());
    }

    // Parse input
    let now = Instant::now();
    let parse_output = parse_fn(buf)?;
    let dur = now.elapsed();

    if verbose {
        eprintln!("{}: parse time: {}.{:09}",
                  PROG_NAME, dur.as_secs(), dur.subsec_nanos());
    }

    // Early exit if parse-only
    if parse_only {
        return Ok(());
    }

    // Pretty print
    let now = Instant::now();
    pp_fn(parse_output, indent, terms_per_line);
    let dur = now.elapsed();

    if verbose {
        eprintln!("{}: print time: {}.{:09}",
                  PROG_NAME, dur.as_secs(), dur.subsec_nanos());
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

fn parse_disk_log(buf: Vec<u8>) -> Result<Vec<BertTerm>> {
    let mut parser = parser::Parser::new(buf);
    return parser.parse_disk_log();
}
