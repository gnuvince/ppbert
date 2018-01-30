extern crate rayon;
extern crate ppbert;
#[macro_use] extern crate clap;

use clap::{Arg, App};
use rayon::prelude::*;

use std::io::{self, Read, Write, BufWriter};
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;
use std::time::Instant;

use ppbert::bertterm::BertTerm;
use ppbert::error::BertError;
use ppbert::parser;

const PROG_NAME: &str = "bert-convert";

fn main() {
    let matches = App::new(PROG_NAME)
        .version(crate_version!())
        .author("Vincent Foley")
        .about("Convert bertconf .bert files to rig .bert2 files")
        .arg(Arg::with_name("input_files")
             .value_name("FILES")
             .multiple(true))
        .arg(Arg::with_name("output-dir")
             .help("Selects the output directory")
             .value_name("DIR")
             .default_value(".")
             .short("d")
             .long("--output-dir"))
        .arg(Arg::with_name("verbose")
             .help("Enables verbose mode")
             .short("v")
             .long("--verbose"))
        .get_matches();

    let files: Vec<&str> = match matches.values_of("input_files") {
        Some(files) => files.collect(),
        None => vec!["-"]
    };

    let verbose = matches.is_present("verbose");
    let output_dir: PathBuf = matches.value_of("output-dir")
        .unwrap_or(".")
        .into();

    if !output_dir.exists() {
        eprintln!("{}: directory {:?} does not exist", PROG_NAME, output_dir);
        exit(1);
    }

    let ret_code_sum: i32 = files
        .into_par_iter()
        .map(|file| {
            match handle(file, verbose, &output_dir) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("{}: {}", PROG_NAME, e);
                    1
                }
            }
        })
        .sum();
    exit((ret_code_sum > 0) as i32);
}


fn handle(file: &str, verbose: bool, output_dir: &PathBuf) -> Result<(), BertError> {
    let mut buf: Vec<u8> = Vec::new();
    if file == "-" {
        let mut stdin = io::stdin();
        stdin.read_to_end(&mut buf)?;
    } else {
        let mut f = File::open(file)?;
        f.read_to_end(&mut buf)?;
    }

    let top = Instant::now();
    let mut parser = parser::Parser::new(buf);
    let term = parser.parse()?;
    let dur1 = top.elapsed();

    let top = Instant::now();
    create_bert2_file(term, output_dir)?;
    let dur2 = top.elapsed();
    if verbose {
        eprintln!("{}: {}: Parse time: {}.{:09}s; Dump time: {}.{:09}s",
                  PROG_NAME, file,
                  dur1.as_secs(), dur1.subsec_nanos(),
                  dur2.as_secs(), dur2.subsec_nanos());
    }

    return Ok(());
}


fn create_bert2_file(term: BertTerm, output_dir: &PathBuf) -> Result<(), BertError> {
    // TODO(vfoley): this is easily the ugliest Rust code I've written. Fix!
    if let BertTerm::List(terms) = term {
        for term in terms {
            if let BertTerm::Tuple(tup_terms) = term {
                if tup_terms.len() != 2 {
                    return Err(BertError::NotABertFile);
                }
                if let BertTerm::Atom(ref table_name) = tup_terms[0] {
                    if let BertTerm::List(ref items) = tup_terms[1] {
                        let file_name = format!("{}.bert2", table_name);
                        let path = output_dir.join(file_name);
                        let f = File::create(path)?;
                        let mut stream = BufWriter::new(f);
                        for item in items {
                            let item_bert = item.write_bert();
                            stream.write_all(&usize_to_leb128(item_bert.len()))?;
                            stream.write_all(&item_bert)?;
                        }
                    } else {
                        return Err(BertError::NotABertFile);
                    }
                } else {
                    return Err(BertError::NotABertFile);
                }
            } else {
                return Err(BertError::NotABertFile);
            }
        }
    } else {
        return Err(BertError::NotABertFile);
    }
    return Ok(());
}


fn usize_to_leb128(mut n: usize) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(8);
    while n > 0 {
        let mut x: u8 = (n & 0x7f) as u8;
        x |= ((n > 127) as u8) << 7; // if n > 127 { x |= 0x80; }
        bytes.push(x);
        n >>= 7;
    }
    return bytes;
}

#[test]
fn test_usize_to_leb128() {
    assert_eq!(usize_to_leb128(10), vec![0x0A]);
    assert_eq!(usize_to_leb128(300), vec![0xAC, 0x02]);
    assert_eq!(usize_to_leb128(624485), vec![0xE5, 0x8E, 0x26]);
}
