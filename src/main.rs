use std::env;
use std::fs;
use std::io::{self, ErrorKind, Read, Write, BufWriter};
use std::path::Path;
use std::process::exit;
use std::time::{Duration, Instant};

use getopts::Options;

use ppbert::prelude::*;
use ppbert::parsers::*;

const PROG_NAME: &str = "ppbert";

#[derive(Clone, Copy)]
enum ParserChoice {
    ByExtension,
    ForceBert1,
    ForceBert2,
    ForceDiskLog,
}

fn opt_usize(m: &getopts::Matches, opt: &str, default: usize) -> usize {
    match m.opt_get_default(opt, default) {
        Ok(n) => n,
        Err(_) => {
            eprintln!("'{}' must be a number", opt);
            exit(1);
        }
    }
}


fn main() {
    let mut opts = Options::new();
    opts.optflag("V", "version", "display version");
    opts.optflag("h", "help", "display this help");
    opts.optopt("i", "indent", "indent with NUM spaces", "NUM");
    opts.optopt("m", "per-line", "print at most NUM basic terms per line", "NUM");
    opts.optflag("p", "parse", "parse only, not pretty print");
    opts.optflag("1", "bert1", "force ppbert to use regular BERT parser");
    opts.optflag("2", "bert2", "force ppbert to use BERT2 parser");
    opts.optflag("d", "disk-log", "force ppbert to use DiskLog parser");
    opts.optflag("v", "verbose", "show diagnostics on stderr");
    opts.optflag("j", "json", "print as JSON");
    opts.optflag("t", "transform-proplists", "convert proplists to JSON objects");

    let mut matches = match opts.parse(env::args().skip(1)) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}: {}", PROG_NAME, e);
            eprintln!("{}", opts.usage(&format!("{} {}", PROG_NAME, VERSION)));
            exit(1);
        }
    };

    if matches.opt_present("help") {
        println!("{}", opts.usage(&format!("{} {}", PROG_NAME, VERSION)));
        exit(0);
    }

    if matches.opt_present("version") {
        println!("{} {}", PROG_NAME, VERSION);
        exit(0);
    }

    // If no files to process, use stdin.
    if matches.free.is_empty() {
        matches.free.push("-".to_owned());
    }

    let indent_width = opt_usize(&matches, "indent", 2);
    let max_per_line = opt_usize(&matches, "per-line", 6);
    let parse_only = matches.opt_present("parse");
    let json = matches.opt_present("json");
    let transform_proplists = matches.opt_present("transform-proplists");
    let verbose = matches.opt_present("verbose");

    let parser_choice =
        if matches.opt_present("bert1") {
            ParserChoice::ForceBert1
        } else if matches.opt_present("bert2") {
            ParserChoice::ForceBert2
        } else if matches.opt_present("disk-log") {
            ParserChoice::ForceDiskLog
        } else {
            ParserChoice::ByExtension
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
    for file in &matches.free {
        let res = handle_file(file, parse_only, verbose,
                              indent_width, max_per_line,
                              parser_choice, output_fn);
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

fn read_bytes(filename: &str) -> Result<Vec<u8>> {
    if filename == "-" {
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        let mut buf: Vec<u8> = Vec::with_capacity(4096);
        stdin.read_to_end(&mut buf)?;
        return Ok(buf);
    } else {
        let buf = fs::read(filename)?;
        return Ok(buf);
    }
}

fn parser_from_ext(filename: &str, bytes: Vec<u8>) -> Box<dyn Parser> {
    let ext: Option<&str> =
        Path::new(filename)
        .extension()
        .and_then(|x| x.to_str());
    match ext {
        Some("bert") | Some("bert1") => Box::new(Bert1Parser::new(bytes)),
        Some("bert2") => Box::new(Bert2Parser::new(bytes)),
        Some("log") => Box::new(DiskLogParser::new(bytes)),
        _ => {
            eprintln!("{}: cannot find an appropriate parser for {}; using BERT",
                      PROG_NAME, filename);
            Box::new(Bert1Parser::new(bytes))
        },
    }
}

fn handle_file(
    filename: &str,
    parse_only: bool,
    verbose: bool,
    indent: usize,
    terms_per_line: usize,
    parser_choice: ParserChoice,
    pp_fn: fn(BertTerm, usize, usize) -> Result<()>
) -> Result<()> {

    // Read file or stdin into buffer
    let now = Instant::now();
    let bytes = read_bytes(filename)?;
    let read_dur = now.elapsed();

    let mut parser: Box<dyn Parser> = match parser_choice {
        ParserChoice::ForceBert1 => Box::new(Bert1Parser::new(bytes)),
        ParserChoice::ForceBert2 => Box::new(Bert2Parser::new(bytes)),
        ParserChoice::ForceDiskLog => Box::new(DiskLogParser::new(bytes)),
        ParserChoice::ByExtension => parser_from_ext(filename, bytes),
    };

    let mut parse_dur = Duration::new(0, 0);
    let mut pp_dur = Duration::new(0, 0);

    loop {
        let now = Instant::now();
        let next_item = parser.next();
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
        eprintln!("{}: {} read time: {}.{:06} seconds", PROG_NAME, filename, read_dur.as_secs(), read_dur.subsec_micros());
        eprintln!("{}: {} parse time: {}.{:06} seconds", PROG_NAME, filename, parse_dur.as_secs(), parse_dur.subsec_micros());
        if !parse_only {
            eprintln!("{}: {} print time: {}.{:06} seconds", PROG_NAME, filename, pp_dur.as_secs(), pp_dur.subsec_micros());
        }
    }

    return Ok(());
}


/// Outputs a BertTerm to stdout.
fn pp_bert(term: BertTerm, indent_width: usize, terms_per_line: usize) -> Result<()> {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut stdout = BufWriter::new(stdout);
    term.write_as_erlang(&mut stdout, indent_width, terms_per_line)?;
    stdout.flush()?;
    writeln!(&mut stdout, "")?;
    return Ok(());
}

/// Outputs a BertTerm as JSON to stdout.
fn pp_json(term: BertTerm, _: usize, _: usize) -> Result<()> {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut stdout = BufWriter::new(stdout);
    term.write_as_json(&mut stdout, false)?;
    writeln!(&mut stdout, "")?;
    stdout.flush()?;
    return Ok(());
}

/// Outputs a BertTerm as JSON to stdout;
/// Erlang proplists are converted to JSON objects.
fn pp_json_proplist(term: BertTerm, _: usize, _: usize) -> Result<()> {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut stdout = BufWriter::new(stdout);
    term.write_as_json(&mut stdout, true)?;
    writeln!(&mut stdout, "")?;
    stdout.flush()?;
    return Ok(());
}
