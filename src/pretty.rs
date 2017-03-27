use ::bert_parser::BertTerm;
use ::bert_parser::BertTerm::*;

const INDENT_WIDTH: usize = 2;

pub fn print(term: &BertTerm, indent_level: usize) {
    indent(indent_level);
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
            println!("{{");
            let mut is_first = true;
            for term in terms {
                if !is_first { println!(",") }
                print(term, indent_level + 1);
                is_first = false;
            }
            println!();
            indent(indent_level);
            print!("}}");
        }

        List(ref terms) => {
            println!("[");
            let mut is_first = true;
            for term in terms {
                if !is_first { println!(",") }
                print(term, indent_level + 1);
                is_first = false;
            }
            println!();
            indent(indent_level);
            print!("]");
        }

        String(ref bytes) => {
            print!("\"");
            for b in bytes {
                print!("{}", *b as char);
            }
            println!("\"");
        }

        Binary(ref bytes) => {
            if bytes.iter().all(|b| is_printable(*b)) {
                print!("<<\"");
                for b in bytes {
                    print!("{}", *b as char);
                }
                println!("\">>");
            } else {
                print!("<<");
                for b in bytes {
                    print!("{:x}", *b);
                    print!(",");
                }
                println!(">>");
            }
        }
    }
}

fn indent(level: usize) {
    for _ in 0 .. level * INDENT_WIDTH {
        print!(" ");
    }
}

fn is_printable(b: u8) -> bool {
    b >= 0x20 && b <= 0x7e
}
