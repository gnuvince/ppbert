use num::bigint;

#[derive(Debug, PartialEq)]
pub enum BertTerm {
    Int(i32),
    BigInt(bigint::BigInt),
    Float(f64),
    Atom(String),
    Tuple(Vec<BertTerm>),
    List(Vec<BertTerm>),
    String(Vec<u8>),
    Binary(Vec<u8>)
}
