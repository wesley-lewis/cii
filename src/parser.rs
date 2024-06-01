#![allow(warnings)]

use crate::{scanner::TokenType, Token};
use crate::expr::{self, Expr};
use crate::expr::Expr::*;
use crate::scanner::TokenType::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Expr, String> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.comparison()?;
        while self.match_tokens(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let rhs = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(rhs),
                operator,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_tokens(&[Greater, GreaterEqual, Less, LessEqual]) {
            println!("here");
            let op = self.previous();
            let rhs = self.term()?;
            expr = Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_tokens(&[Minus, Plus]) {
            let op = self.previous();
            let rhs = self.factor()?;
            expr = Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_tokens(&[Slash, Star]) {
            let op = self.previous();
            let rhs = self.unary()?;
            expr = Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let op = self.previous();
            let rhs = self.unary()?;
            Ok(Unary {
                operator: op,
                right: Box::from(rhs),
            })
        }else {
            let token = self.peek();
            self.advance();
            Ok(Literal { 
                value: expr::LiteralValue::from_token(token) 
            })
        }
    }


    fn match_token(&mut self, typ: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        } else {
            if &self.peek().token_type == typ {
                self.advance();
                return true;
            }

            false
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peek();

        let result;
        match token.token_type {
            LeftParen =>  {
                let expr = self.expression()?;
                self.consume(RightParen, "Expected ')'");
                result = Grouping {
                    expression: Box::from(expr),
                }
            },
            False | True | Nil | Number | StringLit => {
                // possible bug in future
                // self.advance();
                result = Literal {
                    value: expr::LiteralValue::from_token(token),
                }
            },
            _ => {
                panic!("should not reach over here: {:?}", token);
            }
        }

        // if we get here it means we correctl matched on a token and we need to advance
        // the pointer to consume it
        self.advance();

        Ok(result)
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<(), String> {
        let token = self.peek();
        if token.token_type == token_type {
            self.advance();

            Ok(())
        }else {
            Err(msg.to_string())
        }

    }

    fn match_tokens(&mut self, typs: &[TokenType]) -> bool {
        for typ in typs {
            if self.match_token(typ) {
                return true;
            }
        }

        false
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn peek(&mut self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&mut self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == SemiColon {
                return;
            }

            match self.peek().token_type {
                (Class | Fun | Var | For | If | While | Print | Return) => return,
                _ => {}
            }

            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{scanner::{Token, TokenType}, LiteralValue, Scanner};

    use super::Parser;

    #[test]
    fn test_addition() {
        let one = Token::new(
            TokenType::Number,
            String::from("1"),
            Some(LiteralValue::IntValue(1)),
            0,
        );
        let plus = Token::new( 
            TokenType::Plus,
            String::from("+"),
            None,
            0,
        );
        let two = Token::new(
            TokenType::Number,
            String::from("2"),
            Some(LiteralValue::IntValue(2)),
            0,
        );
        let semicolon = Token::new(
            TokenType::SemiColon,
            String::from(";"),
            None,
            0,
        );
        let mut parser = Parser::new(vec![one, plus, two, semicolon]);
        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr.to_string();
        assert_eq!(string_expr, "(+ 1 2)");
    }

    #[test]
    fn test_comparison() {
        let source = "1 + 2 == 3 + 4";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(scanner.tokens);
        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr.to_string();

        assert_eq!(string_expr, "(== (+ 1 2) (+ 3 4))");
    }

    #[test]
    fn test_eq_with_paren() {
        let source = "1 == (2 + 2)";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(scanner.tokens);
        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr.to_string();

        assert_eq!(string_expr, "(== 1 (group (+ 2 2)))");
    }
}
