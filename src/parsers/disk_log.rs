use crate::prelude::*;
use crate::parsers::*;

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

impl Iterator for DiskLogParser {
    type Item = Result<BertTerm>;

    fn next(&mut self) -> Option<Self::Item> {
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
