extern crate ppbert;

use ppbert::erlang::scanner;

use std::io::{self, Read};

fn main() {
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let mut buf = String::new();
    stdin.read_to_end(&mut buf);

    let chars: Vec<char> = buf.chars().collect();
    let tokens = scanner::scan("<stdin>", chars);

    for token in tokens {
        println!("{:?}", token);
    }
}
