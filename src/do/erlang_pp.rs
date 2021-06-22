use ppbert::pp::utils::*;
use std::io::{self};

use crate::config::Config;
use crate::parser::{Tag, Terms};

pub fn pp(ts: &Terms, config: &Config, w: &mut dyn io::Write) -> io::Result<()> {
    pp_(ts, config, w, &mut 0)?;
    w.write_all(b"\n")
}

// Pre-condition:
//      *pos points to the Tag that needs to be pretty printed.
// Post-condition:
//      *pos points to the next Tag that needs to be pretty printed when it returns.
fn pp_(ts: &Terms, config: &Config, w: &mut dyn io::Write, pos: &mut usize) -> io::Result<()> {
    let curr_term = &ts.tags[*pos];
    *pos += 1;
    match *curr_term {
        Tag::Int(n) => itoa::write(w, n).map(|_| ()),
        Tag::BigInt(ref b) => write!(w, "{}", b),
        Tag::Float(x) => {
            let mut buf = ryu::Buffer::new();
            w.write_all(buf.format(x).as_bytes())
        }
        Tag::Atom { off, len } => {
            let off = off as usize;
            let len = len as usize;
            let buf = &ts.bytes[off..off + len];
            w.write_all(buf)
        }
        Tag::Binary { off, len } => {
            w.write_all(b"<<\"")?;
            pp_string(&ts.bytes[off as usize..off as usize + len as usize], w)?;
            w.write_all(b"\">>")
        }
        Tag::String { off, len } => {
            w.write_all(b"\"")?;
            pp_string(&ts.bytes[off as usize..off as usize + len as usize], w)?;
            w.write_all(b"\"")
        }
        Tag::Tuple(len) => pp_seq(ts, config, len, b"{", b"}", w, pos),
        Tag::List(len) => pp_seq(ts, config, len, b"[", b"]", w, pos),
        Tag::Proplist(len) => {
            if config.transform_proplists {
                let mut sep: &[u8] = b"";
                w.write_all(b"#{")?;
                for _ in 0..len {
                    w.write_all(sep)?;
                    pp_kv(ts, config, w, pos)?;
                    sep = b",";
                }
                w.write_all(b"}")
            } else {
                pp_seq(ts, config, len, b"[", b"]", w, pos)
            }
        }
        Tag::Map(len) => {
            w.write_all(b"{")?;
            let mut sep: &[u8] = b"";
            for _ in 0..len {
                w.write_all(sep)?;
                pp_(ts, config, w, pos)?;
                w.write_all(b":")?;
                pp_(ts, config, w, pos)?;
                sep = b",";
            }
            w.write_all(b"}")
        }
    }
}

fn pp_string(buf: &[u8], w: &mut dyn io::Write) -> io::Result<()> {
    let mut start: usize = 0;
    for (i, b) in buf.iter().enumerate() {
        if !is_printable(*b) && !is_utf8(*b) {
            w.write_all(&buf[start..i])?;
            write!(w, "\\x{:02x}", b)?;
            start = i + 1;
        } else if must_be_escaped(*b) {
            w.write_all(&buf[start..i])?;
            write!(w, "\\{}", *b as char)?;
            start = i + 1;
        }
    }
    w.write_all(&buf[start..])
}

fn pp_seq(
    ts: &Terms,
    config: &Config,
    len: u32,
    open: &[u8],
    close: &[u8],
    w: &mut dyn io::Write,
    pos: &mut usize,
) -> io::Result<()> {
    w.write_all(open)?;
    let mut sep: &[u8] = b"";
    for _ in 0..len {
        w.write_all(sep)?;
        pp_(ts, config, w, pos)?;
        sep = b",";
    }
    w.write_all(close)
}

fn pp_kv(ts: &Terms, config: &Config, w: &mut dyn io::Write, pos: &mut usize) -> io::Result<()> {
    *pos += 1; // Skip the collection tag.
    pp_(ts, config, w, pos)?;
    w.write_all(b"=>")?;
    pp_(ts, config, w, pos)?;
    return Ok(());
}
