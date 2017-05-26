extern crate ppbert;
#[macro_use] extern crate clap;

use std::io::{self, Read, Write};
use std::fs::File;
use std::process::exit;
use std::time::Instant;

use clap::{Arg, App};

use ppbert::parser;
use ppbert::bertterm::{
    BertTerm,
    PrettyPrinter,
    DEFAULT_INDENT_WIDTH,
    DEFAULT_MAX_TERMS_PER_LINE
};
use ppbert::error::Result;


fn main() {
    let matches = App::new("ppbert")
        .version(crate_version!())
        .author("Vincent Foley")
        .about("Pretty print structure encoded in Erlang's External Term Format")
        .arg(Arg::with_name("input_files")
             .value_name("FILES")
             .multiple(true))
        .arg(Arg::with_name("indent_width")
             .value_name("num")
             .short("i")
             .long("--indent-width")
             .takes_value(true))
        .arg(Arg::with_name("max_per_line")
             .value_name("num")
             .short("m")
             .long("--max-terms-per-line")
             .takes_value(true))
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("--verbose"))
        .arg(Arg::with_name("skip_pretty_print")
             .short("s")
             .long("--skip-pretty-print"))
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

    let mut return_code = 0;
    for file in files {
        let now = Instant::now();
        let parse_res = parse_file(file);
        if verbose {
            let dur = now.elapsed();
            let _ = writeln!(&mut io::stderr(),
                             "ppbert: parse time: {}.{}s",
                             dur.as_secs(), dur.subsec_nanos());
        }

        if skip_pretty_print {
            continue;
        }

        let _ = parse_res
            .map(|ref t| {
                let now = Instant::now();
                let pp = PrettyPrinter::new(t, indent_level, max_per_line);
                println!("{}", pp);
                if verbose {
                    let dur = now.elapsed();
                    let _ = writeln!(&mut io::stderr(),
                                     "ppbert: pretty print time: {}.{}s",
                                     dur.as_secs(), dur.subsec_nanos());
                }

            })
            .map_err(|ref e| {
                return_code = 1;
                writeln!(&mut io::stderr(), "ppbert: {}: {}", file, e)
            });
    }
    exit(return_code);
}


fn parse_file(file: &str) -> Result<BertTerm> {
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
