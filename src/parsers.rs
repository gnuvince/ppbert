mod bert1;
mod bert2;
mod disk_log;
mod basic;

pub use bert1::Bert1Parser;
pub use bert2::Bert2Parser;
pub use disk_log::DiskLogParser;

use crate::bertterm::BertTerm;
use crate::error::Result;

pub trait Parser {
    fn next(&mut self) -> Option<Result<BertTerm>>;
}
