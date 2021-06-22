use ppbert::error::{BertError, Result};

use std::fs::File;
use std::io::{self, BufReader, BufWriter, ErrorKind, Read};
use std::path::Path;
use std::process::exit;
use std::time::{Duration, Instant};

mod config;
mod erlang_pp;
mod json_pp;
mod parser;

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

type PpFn = fn(&parser::Terms, &config::Config, &mut dyn io::Write) -> io::Result<()>;

fn main() -> io::Result<()> {
    let mut opts: Opts = gumdrop::parse_args_default_or_exit();

    if opts.version {
        println!("{}", VERSION);
        exit(0);
    }

    let pp_config = config::Config {
        transform_proplists: opts.transform,
        indent: opts.indent,
        short_collection: opts.per_line,
        terminator: if opts.append { "." } else { "" },
    };

    // If no files to process, use stdin.
    if opts.files.is_empty() {
        opts.files.push("-".to_string());
    }

    let parser_choice: Option<parser::ParserNext> = if opts.bert1 {
        Some(parser::BertParser::bert1_next)
    } else if opts.bert2 {
        Some(parser::BertParser::bert2_next)
    } else if opts.disk_log {
        Some(parser::BertParser::disk_log_next)
    } else {
        None
    };

    let pp: PpFn = if opts.json {
        json_pp::pp
    } else {
        erlang_pp::pp
    };

    let mut return_code = 0;

    for file in &opts.files {
        let t = Instant::now();
        let mut buf = Vec::with_capacity(1 << 16);
        read_content(&file, &mut buf)?;
        if opts.verbose {
            eprintln!("{}: {}: read time: {:?}", PROG_NAME, file, t.elapsed());
        }

        if let Err(ref e) = parse_and_print(
            file,
            buf,
            opts.parse,
            opts.verbose,
            parser_choice,
            pp,
            &pp_config,
        ) {
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
        BertError::IoError(ref ioerr) => ioerr.kind() == ErrorKind::BrokenPipe,
        _ => false,
    }
}

fn read_content(filename: &str, buf: &mut Vec<u8>) -> io::Result<()> {
    if filename == "-" {
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        stdin.read_to_end(buf)?;
    } else {
        let f = File::open(filename)?;
        let mut f = BufReader::new(f);
        f.read_to_end(buf)?;
    }
    return Ok(());
}

fn parser_from_ext(filename: &str) -> parser::ParserNext {
    let ext: Option<&str> = Path::new(filename).extension().and_then(|x| x.to_str());
    match ext {
        Some("bert") | Some("bert1") => parser::BertParser::bert1_next,
        Some("bert2") => parser::BertParser::bert2_next,
        //Some("log") => parser::BertParser::disk_log_next,
        _ => {
            eprintln!(
                "{}: cannot find an appropriate parser for {}; using BERT",
                PROG_NAME, filename
            );
            parser::BertParser::bert1_next
        }
    }
}

fn parse_and_print(
    file: &str,
    buf: Vec<u8>,
    parse_only: bool,
    verbose: bool,
    parser_choice: Option<parser::ParserNext>,
    pp: PpFn,
    pp_config: &config::Config,
) -> Result<()> {
    let mut parser = parser::BertParser::new(buf);
    let mut terms = parser::Terms::default();

    let parser_fn = match parser_choice {
        None => parser_from_ext(file),
        Some(f) => f,
    };

    let mut parse_dur = Duration::new(0, 0);
    let mut pp_dur = Duration::new(0, 0);

    loop {
        terms.clear();
        let parse_start = Instant::now();
        let res = parser_fn(&mut parser, &mut terms);
        parse_dur += parse_start.elapsed();
        match res {
            None => break,
            Some(Err(e)) => {
                eprintln!("{}: {}: {}", PROG_NAME, file, e);
                return Err(e);
            }
            Some(Ok(())) => {
                if !parse_only {
                    let stdout = io::stdout();
                    let stdout = stdout.lock();
                    let mut stdout = BufWriter::new(stdout);
                    let pp_start = Instant::now();
                    pp(&terms, &pp_config, &mut stdout)?;
                    pp_dur += pp_start.elapsed();
                }
            }
        }
    }

    if verbose {
        eprintln!("{}: {}: parse time: {:?}", PROG_NAME, file, parse_dur);
        eprintln!("{}: {}: print time: {:?}", PROG_NAME, file, pp_dur);
    }

    return Ok(());
}
