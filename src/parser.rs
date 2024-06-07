use crate::{scanner::TokenType, Token};
use crate::expr::{Expr, LiteralValue};
use crate::expr::Expr::*;
use crate::scanner::TokenType::*;
use crate::stmt::Stmt;

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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = vec![];
        let mut errs = vec![];

        while !self.is_at_end() {
            let stmt = self.declaration();
            match stmt {
                Ok(s) => stmts.push(s),
                Err(e) => {
                    errs.push(e);
                    self.synchronize();
                }
            }
        }

        if errs.len() != 0 {
            return Err(errs.join("\n"));
        }

        Ok(stmts) 
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_token(&Var) {
            match self.var_declaration() {
                Ok(stmt) => Ok(stmt),
                Err(e) => {
                    Err(e)
                }, 
            }
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let token = self.consume(Identifier, "Expected variable name")?;

        let initializer;
        if self.match_token(&Equal) {
            initializer = self.expression()?;
        }else {
            initializer = Expr::Literal { value: LiteralValue::Nil };
        }

        self.consume(SemiColon, "Expected ';' after variable declaration")?;

        Ok( Stmt::Var { name: token, initializer: initializer } )
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&Print) {
            self.print_statement()
        }else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(SemiColon, "Expected ';' after value.")?;
        Ok( Stmt::Print {
            expression: expr,
        })
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(SemiColon, "Expected ';' after value.")?;
        Ok(Stmt::Expression {
            expression: expr,
        })
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.equality()?;

        if self.match_token(&Equal) {
            let _equals = self.previous();
            let value = self.assignment()?;

            match expr {
                Variable { ref name } => {
                    return Ok(Assign {
                        name: name.clone(),
                        value: Box::from(value)
                    });
                }
                _ => return Err("invalid assignment target.".to_string()),
            }
        }

        Ok(expr)
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
            self.primary()
        }
    }

    // fn primary(&mut self) -> Result<Expr, String> {
    //     if self.match_token(&LeftParen) {
    //         let expr = self.expression()?;
    //         self.consume(RightParen, "Expected ')'")?;
    //         Ok(
    //             Grouping {
    //                 expression: Box::from(expr),
    //         })
    //     }else {
    //         let token = self.peek();
    //         self.advance();
    //         Ok(
    //             Literal{
    //                 value: LiteralValue::from_token(token),
    //         })
    //     }
    // }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peek();

        let result;
        match token.token_type {
            LeftParen =>  {
                self.advance();
                let expr = self.expression()?;
                self.consume(RightParen, "Expected ')'")?;
                result = Grouping {
                    expression: Box::from(expr),
                };
            },
            False | True | Nil | Number | StringLit => {
                // possible bug in future
                self.advance();
                result = Literal {
                    value: LiteralValue::from_token(token),
                };
            },
            Identifier => {
                println!("Primary");
                self.advance();
                result = Variable { name: self.previous() }
            }
            _ => {
                return Err("Expected expression".to_string());
            }
        }

        Ok(result)
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

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<Token, String> {
        let token = self.peek();
        if token.token_type == token_type {
            self.advance();
            let token = self.previous();

            Ok(token)
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
                Class | Fun | Var | For | If | While | Print | Return => return,
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
        let print = Token::new(
            TokenType::Print,
            "".to_string(),
            None,
            0,
        );
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
        let mut parser = Parser::new(vec![print, one, plus, two, semicolon]);
        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr[0].to_string();
        println!("test addition: {}", string_expr);
        assert_eq!(string_expr, "(+ 1 2)");
    }

    #[test]
    fn test_comparison() {
        let source = "1 + 2 == 3 + 4;";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(scanner.tokens);
        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr[0].to_string();

        assert_eq!(string_expr, "(== (+ 1 2) (+ 3 4))");
    }

    #[test]
    fn test_eq_with_paren() {
        let source = "1 == (2 + 2);";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(scanner.tokens);
        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr[0].to_string();

        assert_eq!(string_expr, "(== 1 (group (+ 2 2)))");
    }
}
