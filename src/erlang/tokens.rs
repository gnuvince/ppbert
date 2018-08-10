#[derive(Debug)]
pub enum TokenType {
    Eof
}

#[derive(Debug)]
pub struct Token {
    pub ty: TokenType,
    pub lexeme: Option<String>,
    pub line: u32,
    pub col: u32
}
