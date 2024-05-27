#![allow(warnings)]

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self{
            source: source.to_string(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token(); 
        }

        // adding Eof token
        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line_num: self.line,
        });

        Ok(self.tokens.clone())
    }

    // scan one character
    fn scan_token(&mut self) -> Result<Token, String> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            _ => {} 
        }

        todo!()
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.tokens.push(Token {
            token_type,
            line_num: self.line,
            lexeme: "".to_string(),
            literal: None,
        });
    }
    
    fn advance(&mut self) -> char {
        todo!()
    }

    fn is_at_end(&self) -> bool {
        self.current <= self.source.len()
    }
}

#[derive(Debug, Clone)]
pub enum TokenType {
    // Single char tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Start,

    // One or two chars
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,


    Eof,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    IntValue(i64),
    FValue(f64),
    StringValue(String),
    IdentifierValue(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<LiteralValue>,
    line_num: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<LiteralValue>, line_num: usize) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line_num,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}

/* Example Program
 * var test = 0.1;
 * var test2 = test + 0.2;
 */
