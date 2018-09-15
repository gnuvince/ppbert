extern crate ppbert;
#[macro_use] extern crate clap;
extern crate memmap;

use std::convert::AsRef;
use std::io::{self, ErrorKind, Read, Write, BufWriter};
use std::fs;
use std::process::exit;
use std::time::{Duration, Instant};

use clap::{Arg, App};

use ppbert::bertterm::BertTerm;
use ppbert::error::{BertError, Result};
use ppbert::parser;

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
             .default_value("2")
             .takes_value(true))
        .arg(Arg::with_name("max_per_line")
             .help("Prints at most <num> basic terms per line")
             .value_name("num")
             .short("m")
             .long("max-terms-per-line")
             .default_value("5")
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
             .short("t")
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
    let json = matches.is_present("json");
    let transform_proplists = matches.is_present("transform-proplists");

    let parse_fn: fn(&mut parser::Parser) -> Option<Result<BertTerm>> =
        if matches.is_present("bert2") {
            |parser| parser.bert2_next()
        } else if matches.is_present("disk_log") {
            |parser| parser.disk_log_next()
        } else {
            |parser| parser.bert_next()
        };
    let output_fn = match (json, transform_proplists) {
        (true, false)  => pp_json,
        (true, true)   => pp_json_proplist,
        (false, false) => pp_bert,
        (false, true)  => {
            eprintln!("{}: --transform-proplists is only valid with the --json flag", PROG_NAME);
            pp_bert
        }
    };

    let mut return_code = 0;
    for file in files {
        let res = handle_file(file, parse_only, verbose,
                              indent_level, max_per_line,
                              parse_fn, output_fn);
        match res {
            Ok(()) => (),
            Err(ref e) => {
                if broken_pipe(e) {
                    break;
                }
                return_code = 1;
                eprintln!("{}: {}: {}", PROG_NAME, file, e);
            }
        }
    }
    exit(return_code);
}


fn broken_pipe(err: &BertError) -> bool {
    match *err {
        BertError::IoError(ref ioerr) =>
            ioerr.kind() == ErrorKind::BrokenPipe,
        _ => false
    }
}


fn handle_file(
    file: &str,
    parse_only: bool,
    verbose: bool,
    indent: usize,
    terms_per_line: usize,
    parse_fn: fn(&mut parser::Parser) -> Option<Result<BertTerm>>,
    pp_fn: fn(BertTerm, usize, usize) -> Result<()>
) -> Result<()> {

    // Read file or stdin into buffer
    let now = Instant::now();
    let buf: Box<AsRef<[u8]>> =
        if file == "-" {
            let stdin = io::stdin();
            let mut stdin = stdin.lock();
            let mut buf: Vec<u8> = Vec::with_capacity(4096);
            stdin.read_to_end(&mut buf)?;
            Box::new(buf)
        } else {
            let fd = fs::File::open(file)?;
            let mmap = unsafe { memmap::Mmap::map(&fd)? };
            Box::new(mmap)
        };
    let read_dur = now.elapsed();

    let mut parser = parser::Parser::new((*buf).as_ref());

    let mut parse_dur = Duration::new(0, 0);
    let mut pp_dur = Duration::new(0, 0);

    loop {
        let now = Instant::now();
        let next_item = parse_fn(&mut parser);
        parse_dur += now.elapsed();

        match next_item {
            None => { break; }
            Some(Err(e)) => { return Err(e); }
            Some(Ok(t)) => {
                if !parse_only {
                    let now = Instant::now();
                    pp_fn(t, indent, terms_per_line)?;
                    pp_dur += now.elapsed();
                }
            }
        }
    }

    if verbose {
        eprintln!("{}: {} read time: {}.{:06} seconds", PROG_NAME, file, read_dur.as_secs(), read_dur.subsec_micros());
        eprintln!("{}: {} parse time: {}.{:06} seconds", PROG_NAME, file, parse_dur.as_secs(), parse_dur.subsec_micros());
        if !parse_only {
            eprintln!("{}: {} print time: {}.{:06} seconds", PROG_NAME, file, pp_dur.as_secs(), pp_dur.subsec_micros());
        }
    }

    return Ok(());
}


/// Outputs a BertTerm to stdout.
fn pp_bert(term: BertTerm, indent_width: usize, terms_per_line: usize) -> Result <()> {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut stdout = BufWriter::new(stdout);
    term.write_as_erlang(&mut stdout, indent_width, terms_per_line)?;
    writeln!(&mut stdout, "")?;
    return Ok(());
}

/// Outputs a BertTerm as JSON to stdout.
fn pp_json(term: BertTerm, _: usize, _: usize) -> Result <()> {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut stdout = BufWriter::new(stdout);
    term.write_as_json(&mut stdout, false)?;
    writeln!(&mut stdout, "")?;
    return Ok(());
}

/// Outputs a BertTerm as JSON to stdout;
/// Erlang proplists are converted to JSON objects.
fn pp_json_proplist(term: BertTerm, _: usize, _: usize) -> Result <()> {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut stdout = BufWriter::new(stdout);
    term.write_as_json(&mut stdout, true)?;
    writeln!(&mut stdout, "")?;
    return Ok(());
}
