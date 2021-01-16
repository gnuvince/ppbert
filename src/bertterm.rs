use num_bigint::BigInt;

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
            BertTerm::List(_) | BertTerm::Tuple(_) | BertTerm::Map(_, _) => false,
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
                _ => false,
            }
        }

        fn is_proplist_entry(t: &BertTerm) -> bool {
            match *t {
                BertTerm::Tuple(ref elems) => is_proplist_tuple(elems),
                _ => false,
            }
        }

        match *self {
            BertTerm::List(ref elems) => elems.iter().all(|e| is_proplist_entry(e)),
            _ => false,
        }
    }
}
