extern crate ppbert;

use ppbert::parser;
use ppbert::bertterm::BertTerm;
use ppbert::error::Result;

fn p(bytes: &[u8]) -> Result<Vec<BertTerm>> {
    let mut parser = parser::Parser::new(bytes);
    parser.parse_bert2()
}

#[test]
fn zero_terms() {
    assert!(p(&[]).is_ok());
}

#[test]
fn one_term() {
    // ppbert ignores the length.
    assert!(p(&[0, 131, 97, 0]).is_ok());
    assert!(p(&[0, 130, 97, 0]).is_err());
}

#[test]
fn two_terms() {
    // ppbert ignores the length.
    assert!(p(&[0, 131, 97, 0,
                1, 131, 97, 0]).is_ok());
    assert!(p(&[0, 130, 97, 0,
                1, 131, 97, 0]).is_err());
    assert!(p(&[0, 131, 97, 0,
                1, 130, 97, 0]).is_err());
    assert!(p(&[0, 130, 97, 0,
                1, 130, 97, 0]).is_err());
}
