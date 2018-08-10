use erlang::tokens::{Token, TokenType};
use error::{BertError, Result};

enum State {
    Ready,
    SkipWhitespace,
    Eof,
}

struct Scanner {
    filename: String,
    src: Vec<char>,
    pos: usize,

    state: State,

    tokens: Vec<Token>,

    curr_line: u32,
    curr_col: u32,

    saved_line: u32,
    saved_col: u32,
}


pub fn scan(filename: &str, src: Vec<char>) -> Result<Vec<Token>> {
    let mut scanner = Scanner {
        filename: filename.to_owned(),
        src: src,
        pos: 0,
        state: State::Ready,
        tokens: Vec::new(),
        curr_line: 1,
        curr_col: 1,
        saved_line: 0,
        saved_col: 0,
    };
    scanner.run()?;
    return Ok(scanner.tokens);
}


impl Scanner {
    fn run(&mut self) -> Result<()> {
        loop {
            match self.state {
                State::Ready => {
                    self.saved_line = self.curr_line;
                    self.saved_col = self.curr_col;

                    match self.peek() {
                        c if c.is_whitespace() => { self.state = State::SkipWhitespace; }

                        '\x00' => { self.state = State::Eof; }
                        _ => { return self.err("unknown character"); }
                    }
                }

                State::SkipWhitespace => match self.peek() {
                    c if c.is_whitespace() => { self.advance(); }
                    _ => { self.state = State::Ready; }
                }

                State::Eof => match self.peek() {
                    '\x00' => {
                        self.tok(TokenType::Eof);
                        break;
                    }
                    _ => { return self.err("expected Eof"); }
                }
            }
        }
        return Ok(());
    }

    fn err(&self, msg: &str) -> Result<()> {
        Err(BertError::Erlang(self.filename.clone(), self.curr_line, self.curr_col, msg.to_owned()))
    }

    fn peek(&self) -> char {
        if self.pos >= self.src.len() {
            0 as char
        } else {
            self.src[self.pos]
        }
    }

    fn advance(&mut self) {
        if self.peek() == '\n' {
            self.curr_line += 1;
            self.curr_col = 1;
        } else {
            self.curr_col += 1;
        }
        self.pos += 1;
    }

    fn tok(&mut self, ty: TokenType) {
        self.tokens.push(Token {
            ty: ty,
            lexeme: None,
            line: self.curr_line,
            col: self.curr_col
        });
    }
}
