use crate::prelude::*;
use crate::parsers::*;

pub struct Bert1Parser {
    basic_parser: BasicParser
}

impl Bert1Parser {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            basic_parser: BasicParser::new(bytes),
        }
    }
}

impl Parser for Bert1Parser {
    fn next(&mut self) -> Option<Result<BertTerm>> {
        if self.basic_parser.eof() {
            return None;
        }
        let result =
            self.basic_parser.magic_number()
            .and_then(|_| self.basic_parser.bert_term());
        return Some(result);
    }
}
