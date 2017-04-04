use ::bert_parser::BertTerm;
use ::bert_parser::BertTerm::*;

const INDENT_WIDTH: usize = 2;
const MAX_TERMS_PER_LINE: usize = 4;

pub fn print(term: &BertTerm, indent_level: usize) {
    match *term {
        Int(n) => {
            print!("{}", n);
        }

        BigInt(ref n) => {
            print!("{}", n);
        }

        Float(f) => {
            print!("{}", f);
        }

        Atom(ref a) => {
            print!("{}", a);
        }

        Tuple(ref terms) => {
            print_collection(terms, indent_level, '{', '}');
        }

        List(ref terms) => {
            print_collection(terms, indent_level, '[', ']');
        }

        String(ref bytes) => {
            print!("\"");
            for b in bytes {
                print!("{}", *b as char);
            }
            print!("\"");
        }

        Binary(ref bytes) => {
            if bytes.iter().all(|b| is_printable(*b)) {
                print!("<<\"");
                for b in bytes {
                    print!("{}", *b as char);
                }
                print!("\">>");
            } else {
                print!("<<");
                for b in bytes {
                    print!("{}", *b);
                    print!(",");
                }
                print!(">>");
            }
        }
    }
}

fn print_collection(terms: &[BertTerm], level: usize, open: char, close: char) {
    let is_single_line = print_on_single_line(terms);

    print!("{}", open);

    let mut is_first = true;
    for t in terms {
        if !is_first { print!(", "); }
        if !is_single_line {
            println!();
            indent(level + 1);
            print(t, level + 1);
        } else {
            print(t, level + 1);
        }
        is_first = false;
    }

    if !is_single_line { println!(); indent(level); }
    print!("{}", close);
}

fn indent(level: usize) {
    for _ in 0 .. level * INDENT_WIDTH {
        print!(" ");
    }
}

fn is_printable(b: u8) -> bool {
    b >= 0x20 && b <= 0x7e
}

fn is_basic(t: &BertTerm) -> bool {
    match *t {
        Int(_) | BigInt(_) | Float(_) | Atom(_) | String(_) | Binary(_) => true,
        List(_) | Tuple(_) => false
    }
}

fn print_on_single_line(terms: &[BertTerm]) -> bool {
    terms.len() <= MAX_TERMS_PER_LINE &&
        terms.iter().all(is_basic)
}
