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
        let mut errors = vec![];
        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(_) => {},
                Err(msg) => errors.push(msg),
            }
        }

        // adding Eof token
        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line_num: self.line,
        });

        if errors.len() > 0 {
            let mut joined = "".to_string();
            errors.iter().map(|msg| {
                joined.push_str(&msg);
                joined.push_str("\n");
            });

            return Err(joined);
        }
        Ok(self.tokens.clone())
    }

    // scan one character
    fn scan_token(&mut self) -> Result<(), String> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::SemiColon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let token = if self.char_match('=') {
                    TokenType::BangEqual
                }else {
                    TokenType::Bang
                };
                self.add_token(token);
            },
            '=' =>  {
                let token = if self.char_match('=') {
                    TokenType::EqualEqual
                }else {
                    TokenType::Equal
                };
                self.add_token(token);
            },
            '<' => {
                let token = if self.char_match('=') {
                    TokenType::LessEqual
                }else {
                    TokenType::Less
                };

                self.add_token(token);
            },
            
            '>' => {
                let token = if self.char_match('=') {
                    TokenType::GreaterEqual
                }else {
                    TokenType::Greater
                };

                self.add_token(token);
            },
            '/' => {
                if self.char_match('/') {
                    loop {
                        if self.peek() == '\n' || self.is_at_end() {
                            break;
                        }
                        self.advance();
                    }
                }else {
                    self.add_token(TokenType::Slash);
                }
            },
            ' ' | '\r' | '\t' => {},
            '\n' => self.line += 1,
           '+' => self.add_token(TokenType::Plus),
            '-' => self.add_token(TokenType::Minus),
            _ => return Err(format!("unrecognised char at line {}: {}", self.line, c)),
        }

        Ok(())
    }

    fn char_match(&mut self, ch: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.as_bytes()[self.current] as char != ch {
            return false;
        }else {
            self.current += 1;
            return true;
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_lit(token_type, None);
    }

    fn add_token_lit(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token {
            token_type,
            literal,
            line_num: self.line,
            lexeme: text,
        });
    }
    
    fn advance(&mut self) -> char {
        let c = self.source.as_bytes()[self.current];
        self.current += 1;

        c as char
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source.as_bytes()[self.current] as char
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    Star,

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

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    IntValue(i64),
    FValue(f64),
    StringValue(String),
    IdentifierValue(String),
}

#[derive(Debug, Clone, PartialEq)]
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
#[cfg(test)]
mod tests {
    use crate::Scanner;
    use crate::TokenType;

    #[test]
    fn handle_one_char_token() {
        let source = "(( )) }{";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        assert_eq!(scanner.tokens.len(), 7);
        assert_eq!(scanner.tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(scanner.tokens[1].token_type, TokenType::LeftParen);
        assert_eq!(scanner.tokens[2].token_type, TokenType::RightParen);
        assert_eq!(scanner.tokens[3].token_type, TokenType::RightParen);
        assert_eq!(scanner.tokens[4].token_type, TokenType::RightBrace);
        assert_eq!(scanner.tokens[5].token_type, TokenType::LeftBrace);
        assert_eq!(scanner.tokens[6].token_type, TokenType::Eof);
    }

    #[test]
    fn handle_two_char_token() {
        let source = "! != == >=";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        assert_eq!(scanner.tokens.len(), 5);
        assert_eq!(scanner.tokens[0].token_type, TokenType::Bang);
        assert_eq!(scanner.tokens[1].token_type, TokenType::BangEqual);
        assert_eq!(scanner.tokens[2].token_type, TokenType::EqualEqual);
        assert_eq!(scanner.tokens[3].token_type, TokenType::GreaterEqual);
        assert_eq!(scanner.tokens[4].token_type, TokenType::Eof);
    }
}
