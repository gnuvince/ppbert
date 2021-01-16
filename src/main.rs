use std::env;
use std::fs;
use std::io::{self, BufWriter, ErrorKind, Read};
use std::path::Path;
use std::process::exit;
use std::time::{Duration, Instant};

use ppbert::parser::*;
use ppbert::pp::*;
use ppbert::prelude::*;

const PROG_NAME: &str = env!("CARGO_BIN_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(gumdrop::Options)]
struct Opts {
    #[options(help = "display help")]
    help: bool,

    #[options(short = "V", long = "version", help = "display version")]
    version: bool,

    #[options(short = "v", long = "verbose", help = "show diagnostics on stderr")]
    verbose: bool,

    #[options(short = "p", long = "parse", help = "only parse, do not pretty print")]
    parse: bool,

    #[options(
        short = "1",
        long = "bert1",
        help = "force ppbert to use regular BERT parser"
    )]
    bert1: bool,

    #[options(short = "2", long = "bert2", help = "force ppbert to use BERT2 parser")]
    bert2: bool,

    #[options(
        short = "d",
        long = "disk-log",
        help = "force ppbert to use disk-log parser"
    )]
    disk_log: bool,

    #[options(
        short = "i",
        long = "indent",
        help = "indent with NUM space",
        meta = "NUM",
        default = "2"
    )]
    indent: usize,

    #[options(
        short = "m",
        long = "per-line",
        help = "print at most NUM basic terms per line",
        meta = "NUM",
        default = "6"
    )]
    per_line: usize,

    #[options(
        short = ".",
        long = "append-period",
        help = "append a period to Erlang terms (useful for loading with file:consult/1)"
    )]
    append: bool,

    #[options(short = "j", long = "json", help = "pretty print as JSON")]
    json: bool,

    #[options(
        short = "t",
        long = "transform-proplists",
        help = "transform Erlang proplists into JSON objects"
    )]
    transform: bool,

    #[options(short = "b", long = "bert", help = "print as BERT")]
    bert: bool,

    #[options(help = "files to process", free)]
    files: Vec<String>,
}

fn main() {
    let mut opts: Opts = gumdrop::parse_args_default_or_exit();

    if opts.version {
        println!("{}", VERSION);
        exit(0);
    }

    // If no files to process, use stdin.
    if opts.files.is_empty() {
        opts.files.push("-".to_string());
    }

    let parser_choice: Option<ParserNext> = if opts.bert1 {
        Some(BertParser::bert1_next)
    } else if opts.bert2 {
        Some(BertParser::bert2_next)
    } else if opts.disk_log {
        Some(BertParser::disk_log_next)
    } else {
        None
    };

    let pp: Box<dyn PrettyPrinter> = if opts.json {
        Box::new(JsonPrettyPrinter::new(opts.transform))
    } else if opts.bert {
        Box::new(BertWriter::new())
    } else {
        let terminator = if opts.append { "." } else { "" };
        Box::new(ErlangPrettyPrinter::new(
            opts.indent,
            opts.per_line,
            terminator,
        ))
    };

    let mut return_code = 0;
    for file in &opts.files {
        if let Err(ref e) = handle_file(file, opts.parse, opts.verbose, parser_choice, &*pp) {
            if broken_pipe(e) {
                break;
            }
            return_code = 1;
            eprintln!("{}: {:?}: {}", PROG_NAME, file, e);
        }
    }
    exit(return_code);
}

fn broken_pipe(err: &BertError) -> bool {
    match *err {
        BertError::IoError(ref ioerr) => ioerr.kind() == ErrorKind::BrokenPipe,
        _ => false,
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
    let ext: Option<&str> = Path::new(filename).extension().and_then(|x| x.to_str());
    match ext {
        Some("bert") | Some("bert1") => BertParser::bert1_next,
        Some("bert2") => BertParser::bert2_next,
        Some("log") => BertParser::disk_log_next,
        _ => {
            eprintln!(
                "{}: cannot find an appropriate parser for {}; using BERT",
                PROG_NAME, filename
            );
            BertParser::bert1_next
        }
    }
}

fn handle_file(
    filename: &str,
    parse_only: bool,
    verbose: bool,
    parser_choice: Option<ParserNext>,
    pp: &dyn PrettyPrinter,
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
