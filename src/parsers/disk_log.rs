use crate::bertterm::BertTerm;
use crate::error::Result;
use crate::parsers::Parser;
use crate::parsers::basic::BasicParser;

pub struct DiskLogParser {
    basic_parser: BasicParser
}

impl DiskLogParser {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            basic_parser: BasicParser::new(bytes),
        }
    }
}

impl Parser for DiskLogParser {
    fn next(&mut self) -> Option<Result<BertTerm>> {
        if self.basic_parser.eof() {
            return None;
        }
        let result =
            self.basic_parser.disk_log_magic()
            .and_then(|_| self.basic_parser.disk_log_opened_status())
            .and_then(|_| self.basic_parser.disk_log_term());
        return Some(result);
    }
}
