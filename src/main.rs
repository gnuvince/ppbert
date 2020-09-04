use std::env;
use std::fs;
use std::io::{self, ErrorKind, Read, BufWriter};
use std::path::Path;
use std::process::exit;
use std::time::{Duration, Instant};

use getopts::Options;

use ppbert::prelude::*;
use ppbert::parser::*;
use ppbert::pp::*;

const PROG_NAME: &str = "ppbert";

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
    opts.optflag("b", "as-bert", "print as BERT");
    opts.optflag("", "append-period", "append a period to terms (useful for loading terms with file:consult/1)");

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
    let as_bert = matches.opt_present("as-bert");
    let transform_proplists = matches.opt_present("transform-proplists");
    let verbose = matches.opt_present("verbose");
    let append_period = matches.opt_present("append-period");

    let parser_choice: Option<ParserNext> =
        if matches.opt_present("bert1") {
            Some(BertParser::bert1_next)
        } else if matches.opt_present("bert2") {
            Some(BertParser::bert2_next)
        } else if matches.opt_present("disk-log") {
            Some(BertParser::disk_log_next)
        } else {
            None
        };

    let pp: Box<dyn PrettyPrinter> =
        if json {
            Box::new(JsonPrettyPrinter::new(transform_proplists))
        } else if as_bert {
            Box::new(BertWriter::new())
        } else {
            let terminator = if append_period { "." } else { "" };
            Box::new(ErlangPrettyPrinter::new(indent_width, max_per_line, terminator))
        };

    let mut return_code = 0;
    for file in &matches.free {
        if let Err(ref e) = handle_file(file, parse_only, verbose, parser_choice, &pp) {
            if broken_pipe(e) {
                break;
            }
            return_code = 1;
            eprintln!("{}: {}: {}", PROG_NAME, file, e);
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

fn parser_from_ext(filename: &str) -> ParserNext {
    let ext: Option<&str> =
        Path::new(filename)
        .extension()
        .and_then(|x| x.to_str());
    match ext {
        Some("bert") | Some("bert1") => BertParser::bert1_next,
        Some("bert2") => BertParser::bert2_next,
        Some("log") => BertParser::disk_log_next,
        _ => {
            eprintln!("{}: cannot find an appropriate parser for {}; using BERT",
                      PROG_NAME, filename);
            BertParser::bert1_next
        },
    }
}

fn handle_file(
    filename: &str,
    parse_only: bool,
    verbose: bool,
    parser_choice: Option<ParserNext>,
    pp: &Box<dyn PrettyPrinter>,
) -> Result<()> {
    // Read file or stdin into buffer
    let now = Instant::now();
    let bytes = read_bytes(filename)?;
    let read_dur = now.elapsed();
    let mut parser = BertParser::new(bytes);

    let parser_next: ParserNext = match parser_choice {
        Some(f) => f,
        None => parser_from_ext(filename),
    };

    let mut parse_dur = Duration::new(0, 0);
    let mut pp_dur = Duration::new(0, 0);

    loop {
        let now = Instant::now();
        let term = match parser_next(&mut parser) {
            Some(term) => term?,
            None => break,
        };
        parse_dur += now.elapsed();
        if !parse_only {
            let now = Instant::now();
            let stdout = BufWriter::new(io::stdout());
            pp.write(&term, Box::new(stdout))?;
            pp_dur += now.elapsed();
        }
    }

    if verbose {
        eprintln!("{}: {} read time: {:?}", PROG_NAME, filename, read_dur);
        eprintln!("{}: {} parse time: {:?}", PROG_NAME, filename, parse_dur);
        if !parse_only {
            eprintln!("{}: {} print time: {:?}", PROG_NAME, filename, pp_dur);
        }
    }

    return Ok(());
}
