use crate::bertterm::BertTerm;
use crate::error::Result;
use crate::parsers::Parser;
use crate::parsers::basic::BasicParser;

pub struct Bert1Parser {
    basic_parser: BasicParser
}

impl Bert1Parser {
    pub fn new() -> Self {
        Bert1Parser {
            basic_parser: BasicParser::new(vec![])
        }
    }
}

impl Parser for Bert1Parser {
    fn set_input(&mut self, bytes: Vec<u8>) {
        self.basic_parser = BasicParser::new(bytes);
    }

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
