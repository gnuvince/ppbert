pub mod bert;
pub mod erlang;
pub mod json;
pub mod utils;

pub use bert::BertWriter;
pub use erlang::ErlangPrettyPrinter;
pub use json::JsonPrettyPrinter;
pub use utils::*;

use crate::prelude::*;
use std::io;

pub trait PrettyPrinter {
    fn write(&self, term: &BertTerm, w: Box<dyn io::Write>) -> Result<()>;
}
