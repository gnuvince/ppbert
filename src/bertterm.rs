use std::io;

use num_bigint::{BigInt};

use crate::prelude::*;
use crate::pp::*;

/// A enum representing a BertTerm
#[derive(Debug, PartialEq)]
pub enum BertTerm {
    /// The empty list
    Nil,

    /// A signed 32-bit integer
    Int(i32),

    /// A signed arbitrary-size integer
    BigInt(BigInt),

    /// A double-precision floating point number
    Float(f64),

    /// An atom
    Atom(String),

    /// A latin-1-encoded string
    String(Vec<u8>),

    /// An array of bytes
    Binary(Vec<u8>),

    /// A container for a fixed number of elements
    Tuple(Vec<BertTerm>),

    /// A container for an arbitrary number of elements
    List(Vec<BertTerm>),

    /// A container for key-to-value pairs
    Map(Vec<BertTerm>, Vec<BertTerm>),

}

impl BertTerm {
    /// Lists, tuples, and maps are not basic terms;
    /// everything else is.
    pub fn is_basic(&self) -> bool {
        match *self {
            BertTerm::Int(_)
            | BertTerm::BigInt(_)
            | BertTerm::Float(_)
            | BertTerm::Atom(_)
            | BertTerm::String(_)
            | BertTerm::Binary(_)
            | BertTerm::Nil => true,
            BertTerm::List(_)
            | BertTerm::Tuple(_)
            | BertTerm::Map(_, _) => false
        }
    }

    /// A term is a proplist if it has this shape:
    /// [ {atom|string|binary, term}* ]
    pub fn is_proplist(&self) -> bool {
        fn is_proplist_tuple(elems: &[BertTerm]) -> bool {
            match elems {
                [BertTerm::Atom(_), _] => true,
                [BertTerm::String(_), _] => true,
                [BertTerm::Binary(_), _] => true,
                _ => false
            }
        }

        fn is_proplist_entry(t: &BertTerm) -> bool {
            match *t {
                BertTerm::Tuple(ref elems) => is_proplist_tuple(elems),
                _ => false
            }
        }

        match *self {
            BertTerm::List(ref elems) =>
                elems.iter().all(|e| is_proplist_entry(e)),
            _ => false
        }
    }

    /// Writes a `BertTerm` into `W` using Erlang syntax.
    /// The output is indented and printed over multiple lines.
    ///
    /// - `indent_width`:
    ///     how many spaces to use for indentation
    /// - `max_terms_per_line`:
    ///     a list or a tuple made of basic terms, it may be be
    ///     printed on a single line of the number of elements does
    ///     not exceed this limit.
    pub fn write_as_erlang<W: io::Write>(
        &self,
        w: &mut W,
        indent_width: usize,
        max_terms_per_line: usize)
        -> Result<()>
    {
        ErlangPrettyPrinter::new(indent_width, max_terms_per_line)
            .write(&self, w)?;
        Ok(())
    }

    /// Writes a `BertTerm` into `W` using JSON syntax.
    /// The output is not indented and is printed on a single line.
    ///
    /// - `transform_proplists`:
    ///     Erlang proplists are sometimes used in place of maps.
    ///     To make the output of `write_as_json` easier to manipulate
    ///     in a tool like `jq`, the parameter `transform_prolists` is
    ///     given the value `true` and the proplists will be output as
    ///     JSON objects.
    pub fn write_as_json<W: io::Write>
        (&self, w: &mut W, transform_prolists: bool) -> Result<()>
    {
        JsonPrettyPrinter::new(transform_prolists)
            .write(&self, w)?;
        Ok(())
    }

    /// Writes a `BertTerm` into `W` encoded using Erlang's [External Term Format].
    ///
    /// [External Term Format]: http://erlang.org/doc/apps/erts/erl_ext_dist.html
    pub fn write_as_bert<W: io::Write>(&self, w: &mut W) -> Result<()> {
        BertWriter::new()
            .write(&self, w)?;
        return Ok(());
    }
}
