use std::collections::HashMap;

fn is_digit(ch: char) -> bool {
    return ch as u8 >= '0' as u8 && ch as u8 <= '9' as u8;
}

fn is_alpha(ch: char) -> bool {
    let uch = ch as u8;
    (uch >= 'a' as u8 && uch <= 'z' as u8) || 
        (uch >= 'A' as u8 && uch <= 'Z' as u8) ||
        (uch == '_' as u8)
}

fn is_alpha_numeric(ch: char) -> bool {
    let uch = ch as u8;
    if is_digit(ch) || is_alpha(ch) {
        return true;
    }

    false
}

fn get_keywords_hashmap() -> HashMap<&'static str, TokenType> {
    HashMap::from([
        ("and", TokenType::And),
        ("or", TokenType::Or),
        ("print", TokenType::Print),
        ("var", TokenType::Var),
        ("while", TokenType::While),
        ("true", TokenType::True),
        ("false", TokenType::False),
        ("fun", TokenType::Fun),
        ("for", TokenType::For),
        ("if", TokenType::If),
        ("nil", TokenType::Nil),
        ("return", TokenType::Return),
        ("super", TokenType::Super),
        ("this", TokenType::This),
        ("class", TokenType::Class),
        ("else", TokenType::Else),
    ])
}

pub struct Scanner {
    source: String,
    pub tokens: Vec<Token>,
    keywords: HashMap<&'static str, TokenType>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self{
            source: source.to_string(),
            tokens: vec![],
            keywords: get_keywords_hashmap(),
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
            for error in errors {
                joined.push_str(&error);
                joined.push_str("\n");
            }

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
            '"' => self.string()?,
            '+' => self.add_token(TokenType::Plus),
            '-' => self.add_token(TokenType::Minus),
            c => {
                if is_digit(c) {
                    self.number()?;
                }else if is_alpha(c) {
                    self.identifier();
                }
                else {
                    return Err(format!("unrecognised char at line {}: {}", self.line, c));
                }
            }
            _ => return Err(format!("unrecognised char at line {}: {}", self.line, c)),
        }

        Ok(())
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let substring = &self.source[self.start..self.current];
        if let Some(t_type) = self.keywords.get(substring) {
            self.add_token(*t_type);
        }else {
            self.add_token(TokenType::Identifier);
        }
    }

    fn number(&mut self) -> Result<(), String> {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }

            let substring = &self.source[self.start .. self.current];
            let value = match substring.parse::<f64>() {
                Ok(v) => v,
                Err(e) => return Err(format!("Couldn't parse number at line {}: {}", self.line, e)),
            };

            self.add_token_lit(TokenType::Number, Some(LiteralValue::FValue(value)));
        } else {
            let substring = &self.source[self.start .. self.current];
            let value = match substring.parse::<i64>() {
                Ok(v) => v,
                Err(e) => return Err(format!("Couldn't parse number at line {}: {}", self.line, e)),
            };

            self.add_token_lit(TokenType::Number, Some(LiteralValue::IntValue(value)));
        }

        Ok(())
    }

    fn string(&mut self) -> Result<(), String> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(String::from("Unterminated string"));
        }

        self.advance();

        let value = &self.source[self.start + 1 .. self.current - 1];

        self.add_token_lit(TokenType::StringLit, Some(LiteralValue::StringValue(value.to_string())));

        Ok(())
    }

    fn char_match(&mut self, ch: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.current).unwrap() != ch {
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
        let text = self.source[self.start .. self.current].to_string();

        self.tokens.push(Token {
            token_type,
            literal,
            line_num: self.line,
            lexeme: text,
        });
    }
    
    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;

        c
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source.chars().nth(self.current).unwrap()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
    StringLit,
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
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub line_num: usize,
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
    use crate::LiteralValue;

    #[test]
    fn handle_one_char_token() {
        let source = "(( )) }{";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

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
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 5);
        assert_eq!(scanner.tokens[0].token_type, TokenType::Bang);
        assert_eq!(scanner.tokens[1].token_type, TokenType::BangEqual);
        assert_eq!(scanner.tokens[2].token_type, TokenType::EqualEqual);
        assert_eq!(scanner.tokens[3].token_type, TokenType::GreaterEqual);
        assert_eq!(scanner.tokens[4].token_type, TokenType::Eof);
    }

    #[test]
    fn handle_string_literal() {
        let source = r#""ABC""#; // escape sequence comes in between while parsing.
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 2);
        assert_eq!(scanner.tokens[0].token_type, TokenType::StringLit);
        match scanner.tokens[0].literal.as_ref().unwrap() {
            LiteralValue::StringValue(val) => assert_eq!(val, "ABC"),
            _ => panic!("incorrect literal type"),
        }
    }

    #[test]
    fn handle_string_unterminated() {
        let source = r#""ABC"#; // escape sequence comes in between while parsing.
        let mut scanner = Scanner::new(source);
        let result = scanner.scan_tokens();

        match result {
            Err(_) => (),
            _ => panic!("should have failed"),
        }
    }

    #[test]
    fn handle_string_lit_multiline() {
        let source = "\"ABC\ndef\"";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        assert_eq!(scanner.tokens.len(), 2);
        match scanner.tokens[0].literal.as_ref().unwrap() {
            LiteralValue::StringValue(val) => assert_eq!(val, "ABC\ndef"),
            _ => panic!("incorrect literal type"),
        }
    }

    #[test]
    fn handle_number() {
        let source = "123.123\n321.5\n45";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 4);
        for i in 0..3 {
            assert_eq!(scanner.tokens[i].token_type, TokenType::Number);
        }

        match scanner.tokens[0].literal.as_ref().unwrap() {
            LiteralValue::FValue(val) => assert_eq!(*val, 123.123),
            _ => panic!("incorrect value"),
        }

        match scanner.tokens[1].literal.as_ref().unwrap() {
            LiteralValue::FValue(val) => assert_eq!(*val, 321.5),
            _ => panic!("incorrect value"),
        }

        match scanner.tokens[2].literal.as_ref().unwrap() {
            LiteralValue::IntValue(val) => assert_eq!(*val, 45),
            _ => panic!("incorrect value"),
        }
        assert_eq!(scanner.tokens[0].token_type, TokenType::Number);
    }

    #[test]
    fn get_identifier() {
        let source = "this_is_a_var = 12;";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        assert_eq!(scanner.tokens.len(), 5);
        assert_eq!(scanner.tokens[0].token_type, TokenType::Identifier);
        assert_eq!(scanner.tokens[1].token_type, TokenType::Equal);
        assert_eq!(scanner.tokens[2].token_type, TokenType::Number);
        assert_eq!(scanner.tokens[3].token_type, TokenType::SemiColon);
        assert_eq!(scanner.tokens[4].token_type, TokenType::Eof);
    }

    #[test]
    fn get_keywords() {
        let source = "var this_is_a_var = 12;\nwhile true { print 3 };";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        assert_eq!(scanner.tokens.len(), 13);

        assert_eq!(scanner.tokens[0].token_type, TokenType::Var);
        assert_eq!(scanner.tokens[1].token_type, TokenType::Identifier);
        assert_eq!(scanner.tokens[2].token_type, TokenType::Equal);
        assert_eq!(scanner.tokens[3].token_type, TokenType::Number);
        assert_eq!(scanner.tokens[4].token_type, TokenType::SemiColon);
        assert_eq!(scanner.tokens[5].token_type, TokenType::While);
        assert_eq!(scanner.tokens[6].token_type, TokenType::True);
        assert_eq!(scanner.tokens[7].token_type, TokenType::LeftBrace);
        assert_eq!(scanner.tokens[8].token_type, TokenType::Print);
        assert_eq!(scanner.tokens[9].token_type, TokenType::Number);
        assert_eq!(scanner.tokens[10].token_type, TokenType::RightBrace);
        assert_eq!(scanner.tokens[11].token_type, TokenType::SemiColon);
    }
}
