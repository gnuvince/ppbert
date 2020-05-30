mod basic;
mod bert1;
mod bert2;
mod disk_log;

pub use basic::BasicParser;
pub use bert1::Bert1Parser;
pub use bert2::Bert2Parser;
pub use disk_log::DiskLogParser;

use crate::prelude::*;

pub type Parser = dyn Iterator<Item = Result<BertTerm>>;
