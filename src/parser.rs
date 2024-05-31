#![allow(warnings)]

use crate::{scanner::TokenType, Token};
use crate::expr::{self, Expr};
use crate::expr::Expr::*;
use crate::scanner::TokenType::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

macro_rules! match_tokens {
    ($parser:ident, $($token:ident),+) => {
        {
            let mut result = false;
            {
                $(
                    result |= $parser.match_token($token);
                )*
            }

            result
        }
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr: Expr = self.comparison();
        let mut matches_eq = self.match_tokens(&[BangEqual, EqualEqual]);
        while matches_eq {
            let operator = self.previous();
            let rhs = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr.clone()),
                right: Box::new(rhs),
                operator: operator,
            };
        }

        expr
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

    fn unary(&mut self) -> Expr {
        let token = self.peek();
        if self.match_token(&LeftParen) {
            let expr = self.expression();
            self.consume(RightParen, "Expected ')'");
            Grouping {
                expression: Box::from(expr),
            }
        }else {
            Literal {
                value: expr::LiteralValue::from_token(token)
            }
        }
    }

    fn primary(&mut self) -> Expr {
        if self.match_token(&LeftParen) {
            let expr = self.expression();
            self.consume(RightParen, "Expected ')'");
            Grouping {
                expression: Box::from(expr),
            }
        }else {
            let token = self.peek();
            self.advance();
            Literal {
                value: expr::LiteralValue::from_token(token),
            }
        }
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) {
        let token = self.peek();
        if token.token_type == token_type {
            self.advance();
        }else {
            panic!("{}", msg);
        }
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_tokens(&[Minus, Plus]) {
            let op = self.previous();
            let rhs = self.factor();
            expr = Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_tokens(&[Slash, Star]) {
            let op = self.previous();
            let rhs = self.unary();
            expr = Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }

        expr
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
    
    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.match_tokens(&[Greater, GreaterEqual, Less, LessEqual]) {
            let op = self.previous();
            let rhs = self.term();
            expr = Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }

        expr
    }
}
