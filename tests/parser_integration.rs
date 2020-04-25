use num_bigint::ToBigInt;

use ppbert::parser;
use ppbert::bertterm::BertTerm;
use ppbert::error::{BertError, Result};

fn p(bytes: &[u8]) -> Result<BertTerm> {
    let mut parser = parser::BasicParser::new(bytes.to_vec());
    let res = parser.bert_next().unwrap();
    return res;
}

#[test]
fn magic_number() {
    assert!(p(&[131, 97, 0]).is_ok());
    assert!(p(&[130, 97, 0]).is_err());
    assert!(match p(&[130, 97, 0]) {
        Err(BertError::InvalidMagicNumber(_)) => true,
        _ => false
    });
}

#[test]
fn small_integer() {
    for i in 0 .. 255_u8 {
        assert!(match p(&[131, 97, i]) {
            Ok(BertTerm::Int(j)) => i as i32 == j,
            _ => false
        });
    }
}

#[test]
fn integer() {
    for i in 0 .. 255_u8 {
        assert!(match p(&[131, 98, 0, 0, 0, i]) {
            Ok(BertTerm::Int(j)) => j == i as i32,
            _ => false
        });

            assert!(match p(&[131, 98, 0, 0, i, 0]) {
            Ok(BertTerm::Int(j)) => j == (i as i32) << 8,
            _ => false
        });

            assert!(match p(&[131, 98, 0, i, 0, 0]) {
            Ok(BertTerm::Int(j)) => j == (i as i32) << 16,
            _ => false
        });

            assert!(match p(&[131, 98, i, 0, 0, 0]) {
            Ok(BertTerm::Int(j)) => j == (i as i32) << 24,
            _ => false
        });
    }
}

#[test]
fn old_float() {
    assert!(match p(b"\x83\x6398.5\x00") {
        Ok(BertTerm::Float(f)) => f == 98.5,
        _ => false
    });

    assert!(match p(b"\x83\x63-23.5") {
        Ok(BertTerm::Float(f)) => f == -23.5,
        _ => false
    });

    assert!(p(b"\x83\x63abc").is_err());
}

#[test]
fn new_float() {
    let pi: f64 = 3.141592;
    let float_as_u64: u64 = pi.to_bits();
    let bytes = &[
        131, 70,
        (float_as_u64 >> 56) as u8 & 0xff_u8,
        (float_as_u64 >> 48) as u8 & 0xff_u8,
        (float_as_u64 >> 40) as u8 & 0xff_u8,
        (float_as_u64 >> 32) as u8 & 0xff_u8,
        (float_as_u64 >> 24) as u8 & 0xff_u8,
        (float_as_u64 >> 16) as u8 & 0xff_u8,
        (float_as_u64 >> 8) as u8 & 0xff_u8,
        (float_as_u64 >> 0) as u8 & 0xff_u8
    ];
    assert!(match p(bytes) {
        Ok(BertTerm::Float(x)) => x == 3.141592,
        _ => false
    });
}

#[test]
fn atom() {
    // 2-byte atom
    assert!(match p(b"\x83\x64\x00\x04abcd") {
        Ok(BertTerm::Atom(ref s)) => s == "abcd",
        _ => false
    });

    // 1-byte atom
    assert!(match p(b"\x83\x73\x04abcd") {
        Ok(BertTerm::Atom(ref s)) => s == "abcd",
        _ => false
    });

    assert!(match p(b"\x83\x73\x04abc") {
        Err(BertError::NotEnoughData { .. }) => true,
        _ => false
    });

    // latin1 (0xe9 = é)
    assert!(match p(b"\x83\x64\x00\x04caf\xe9") {
        Ok(BertTerm::Atom(ref s)) => s == "café",
        _ => false
    });

    assert!(match p(b"\x83\x73\x04caf\xe9") {
        Ok(BertTerm::Atom(ref s)) => s == "café",
        _ => false
    });
}

#[test]
fn atom_utf8() {
    let atom_name = "jérôme";
    let atom_bytes = atom_name.to_string().into_bytes();

    // 2-byte UTF-8 atom
    let mut bert: Vec<u8> = vec![131, 118, 0, atom_bytes.len() as u8];
    bert.extend(&atom_bytes);
    assert!(match p(&bert) {
        Ok(BertTerm::Atom(ref s)) => s == atom_name,
        _ => false
    });

    // 1-byte UTF-8 atom
    let mut bert: Vec<u8> = vec![131, 119, atom_bytes.len() as u8];
    bert.extend(&atom_bytes);
    assert!(match p(&bert) {
        Ok(BertTerm::Atom(ref s)) => s == atom_name,
        _ => false
    });

    // Not enough data
    let mut bert: Vec<u8> = vec![131, 119, (atom_bytes.len() + 1) as u8];
    bert.extend(&atom_bytes);
    assert!(match p(&bert) {
        Err(BertError::NotEnoughData { .. }) => true,
        _ => false
    });
}

#[test]
fn string() {
    assert!(match p(b"\x83\x6b\x00\x06foobar") {
        Ok(BertTerm::String(ref s)) => s == b"foobar",
        _ => false
    });

    // not enough characters
    assert!(match p(b"\x83\x6b\x00\x04foo") {
        Err(BertError::NotEnoughData { .. }) => true,
        _ => false
    });
}

#[test]
fn binary() {
    assert!(match p(b"\x83\x6d\x00\x00\x00\x06foobar") {
        Ok(BertTerm::Binary(ref s)) => s == b"foobar",
        _ => false
    });

    // not enough characters
    assert!(match p(b"\x83\x6d\x00\x00\x00\x04foo") {
        Err(BertError::NotEnoughData { .. }) => true,
        _ => false
    });
}

#[test]
fn nil() {
    assert!(match p(b"\x83\x6a") {
        Ok(BertTerm::Nil) => true,
        _ => false
    });
}

#[test]
fn list() {
    use ppbert::bertterm::BertTerm::*;
    // proper list
    let b = &[
        131, 108, 0, 0, 0, 4,        // four elements
        97, 16,                      // small int (16)
        98, 0, 0, 255, 255,          // int (65535)
        100, 0, 3, b'a', b'b', b'c', // atom (abc)
        107, 0, 3, b'd', b'e', b'f', // string (def)
        106                          // tail
    ];
    match p(b) {
        Ok(List(ref terms)) => {
            assert_eq!(4, terms.len());
            assert!(match terms[0] {
                Int(16) => true,
                _ => false
            });
            assert!(match terms[1] {
                Int(0xffff) => true,
                _ => false
            });
            assert!(match terms[2] {
                Atom(ref s) => s == "abc",
                _ => false
            });
            assert!(match terms[3] {
                String(ref s) => s == b"def",
                _ => false
            });
        }
        _ => assert!(false)
    };

    // improper list
    let b = &[
        131, 108, 0, 0, 0, 4,        // four elements
        97, 16,                      // small int (16)
        98, 0, 0, 255, 255,          // int (65535)
        100, 0, 3, b'a', b'b', b'c', // atom (abc)
        107, 0, 3, b'd', b'e', b'f', // string (def)
        107, 0, 3, b'g', b'h', b'i'  // tail
    ];
    match p(b) {
        Ok(List(ref terms)) => {
            assert_eq!(5, terms.len());
            assert!(match terms[0] {
                Int(16) => true,
                _ => false
            });
            assert!(match terms[1] {
                Int(0xffff) => true,
                _ => false
            });
            assert!(match terms[2] {
                Atom(ref s) => s == "abc",
                _ => false
            });
            assert!(match terms[3] {
                String(ref s) => s == b"def",
                _ => false
            });
            assert!(match terms[4] {
                String(ref s) => s == b"ghi",
                _ => false
            });
        }
        _ => assert!(false)
    };
}

#[test]
fn tuple() {
    use ppbert::bertterm::BertTerm::*;

    // small
    let b = &[
        131, 104, 2,                 // two elements
        97, 16,                      // small int (16)
        100, 0, 3, b'a', b'b', b'c', // atom (abc)
    ];

    match p(b) {
        Ok(Tuple(ref terms)) => {
            assert_eq!(2, terms.len());
            assert!(match terms[0] {
                Int(16) => true,
                _ => false
            });
            assert!(match terms[1] {
                Atom(ref s) => s == "abc",
                _ => false
            });
        }
        _ => assert!(false)
    };


    // large
    let b = &[
        131, 105, 0, 0, 0, 2,        // two elements
        97, 16,                      // small int (16)
        100, 0, 3, b'a', b'b', b'c', // atom (abc)
    ];

    match p(b) {
        Ok(Tuple(ref terms)) => {
            assert_eq!(2, terms.len());
            assert!(match terms[0] {
                Int(16) => true,
                _ => false
            });
            assert!(match terms[1] {
                Atom(ref s) => s == "abc",
                _ => false
            });
        }
        e => { println!("{:?}", e); assert!(false) }
    };
}

#[test]
fn bigint() {
    // small
    assert!(match p(&[131, 110, 1, 1, 10]) {
        Ok(BertTerm::BigInt(b)) => b == (-10).to_bigint().unwrap(),
        e => { println!("{:?}", e); false }
    });

    // large
    assert!(match p(&[131, 111, 0, 0, 0, 1, 0, 42]) {
        Ok(BertTerm::BigInt(b)) => b == (42).to_bigint().unwrap(),
        e => { println!("{:?}", e); false }
    });
}


#[test]
fn map() {
    let binary = &[
        131,
        116,
        0, 0, 0, 2, // two keys and values
        97, 0,      // keys[0] == 0
        100, 0, 4, b'z', b'e', b'r', b'o',
        97, 42,     // keys[1] == 42
        100, 0, 4, b'h', b'g', b't', b'g'
    ];
    assert!(match p(binary) {
        Ok(BertTerm::Map(ref keys, ref vals)) => {
            keys[0] == BertTerm::Int(0) &&
                keys[1] == BertTerm::Int(42) &&
                vals[0] == BertTerm::Atom("zero".to_string()) &&
                vals[1] == BertTerm::Atom("hgtg".to_string())
        }
        _ => false
    });
}
