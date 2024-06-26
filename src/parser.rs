use crate::{scanner::TokenType, Token};
use crate::expr::{Expr, LiteralValue};
use crate::expr::Expr::*;
use crate::scanner::TokenType::*;
use crate::stmt::Stmt;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
enum FunctionKind {
    Function,
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
        if self.match_token(Var) {
            self.var_declaration()
        }else if self.match_token(Fun) {
            self.function(FunctionKind::Function)
        }else {
            self.statement()
        }
    }

    fn function(&mut self, kind: FunctionKind) -> Result<Stmt, String> {
        let name = self.consume(Identifier, &format!("Expected {kind:?} name"))?;

        self.consume(LeftParen, &format!("Expected '(' after {kind:?} name"))?;
        let mut params = vec![];
        if !self.check(RightParen) {
            loop {
                if params.len() >= 255 {
                    let location = self.peek().line_num;
                    return Err(format!("Line {location}: can't have more than 255 parameters"));
                }
                
                let param = self.consume(Identifier, "Expected parameter name")?;
                params.push(param);

                if !self.match_token(Comma) {
                    break;
                }
            }
        }
        self.consume(RightParen, "Expected ')' after parameters.")?;
        
        self.consume(LeftBrace, &format!("Expected '{{' before {kind:?} body."))?;

        let body = match self.block_statement()? {
            Stmt::Block { statements } => statements,
            _ => return Err(format!("Expected body for {kind:?}")),
        };

        Ok(Stmt::Function {
            name,
            params,
            body,
        })
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let token = self.consume(Identifier, "Expected variable name")?;

        let initializer;
        if self.match_token(Equal) {
            initializer = self.expression()?;
        }else {
            initializer = Expr::Literal { value: LiteralValue::Nil };
       }

        self.consume(SemiColon, "Expected ';' after variable declaration")?;

        Ok( Stmt::Var { name: token, initializer: initializer } )
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(Print) {
            self.print_statement()
        }else if self.match_token(LeftBrace) {
            self.block_statement()
        }else if self.match_token(If) {
            self.if_statement()
        }else if self.match_token(While) {
            self.while_statement()
        }else if self.match_token(For) {
            self.for_statement()
        }
        else {
            self.expression_statement()
        }
    }
    
    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(LeftParen, "Expected '(' after 'for'.")?;

        // Consumes "SMTHNG ;"
        let initializer;
        if self.match_token(SemiColon) {
            initializer = None;
        }else if self.match_token(Var) {
            let var_decl = self.var_declaration()?;
            initializer = Some(var_decl);
        }else {
            let expr = self.expression_statement()?;
            initializer = Some(expr);
        }

        // Consumes "SMTHNG? ;"
        let condition;
        if !self.check(SemiColon) {
            let expr = self.expression()?;
            condition = Some(expr);
        }else {
            condition = None;
        }
        self.consume(SemiColon, "Expected ';' after loop condition")?;

        let increment;
        if !self.check(SemiColon) {
            let expr = self.expression()?;
            increment = Some(expr);
        }else {
            increment = None;
        }
        self.consume(RightParen, "Expected ')' after for clauses")?;

        let mut body = self.statement()?;
        if let Some(inc) = increment {
            body = Stmt::Block {
                statements: vec![
                    Box::new(body), 
                    Box::new(Stmt::Expression { expression: inc }),
                ],
            };
        }
        
        let cond;
        match condition {
            None => cond = Expr::Literal { value: LiteralValue::True },
            Some(c) => cond = c,
        }
        body = Stmt::WhileStmt {
            condition: cond,
            body: Box::new(body),
        };

        if let Some(init) = initializer {
            body = Stmt::Block {
                statements: vec![
                    Box::new(init),
                    Box::new(body),
                ],
            };
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(LeftParen, "Expected '('.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expected ')'.")?;
        let body = self.statement()?;

        Ok(Stmt::WhileStmt { 
            condition, 
            body: Box::from(body) 
        })
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(LeftParen, "Expected '('.")?;
        let predicate = self.expression()?;
        self.consume(RightParen, "Expected ')'")?;

        let then = self.statement()?;

        let els = if self.match_token(Else) {
            let stm = self.statement()?;
            Some(Box::from(stm))
        }else {
            None
        };

        Ok(Stmt::IfStmt { 
            predicate, 
            then: Box::from(then), 
            els 
        })
    }

    fn block_statement(&mut self) -> Result<Stmt, String> {
        let mut statements = vec![];

        while !self.check(RightBrace) && !self.is_at_end() {
            let decl = self.declaration()?;
            statements.push(Box::new(decl));
        }

        self.consume(RightBrace, "Expected '}'.")?;

        Ok(Stmt::Block { 
            statements 
        })
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
        let expr = self.or()?;

        if self.match_token(Equal) {
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

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;
        
        while self.match_token(Or) {
            let operator = self.previous();
            let right = self.and()?;

            expr = Logical {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.match_token(And) {
            let operator = self.previous();
            let right = self.equality()?;

            expr = Logical {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right)
            };
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
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(LeftParen) {
                expr = self.finish_call(expr)?;
            }else {
                break;
            }
        }
        // Apply to arguments

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut arguments = vec![];
        if !self.check(RightParen) {
            loop {
                let arg = self.expression()?;
                arguments.push(arg);
                if arguments.len() >= 255 {
                    let location = self.peek().line_num;
                    return Err(format!("line: {location} cannot have more than 255 arguments"));
                }
                if !self.match_token(Comma) {
                    break;
                }
            }
        }

        self.consume(RightParen, "Expect ')' after function arguments")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
            paren: Token::new(LeftParen, "".to_string(), None, 0),
        })
    }

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
                self.advance();
                result = Variable { name: self.previous() }
            }
            _ => {
                return Err("Expected expression".to_string());
            }
        }

        Ok(result)
    }

    fn check(&mut self, typ: TokenType) -> bool {
        self.peek().token_type == typ
    }

    fn match_token(&mut self, typ: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        } else {
            if self.peek().token_type == typ {
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
            if self.match_token(*typ) {
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
    use crate::Scanner;
    use crate::scanner::{Token, TokenType, LiteralValue};

    use super::Parser;

    #[test]
    fn test_addition() {
        let print = Token::new(
            TokenType::Print,
            "print".to_string(),
            None,
            0,
        );
        let one = Token::new(
            TokenType::Number,
            String::from("1.0"),
            Some(LiteralValue::FValue(1.0)),
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
            String::from("2.0"),
            Some(LiteralValue::FValue(2.0)),
            0,
        );
        let semicolon = Token::new(
            TokenType::SemiColon,
            String::from(";"),
            None,
            0,
        );
        let eof = Token::new(
            TokenType::Eof,
            "".to_string(),
            None,
            0
        );
        let mut parser = Parser::new(vec![print, one, plus, two, semicolon, eof]);
        let statements = parser.parse().unwrap();
        let string_expr = statements[0].to_string();

        assert_eq!("(print (+ 1 2))", string_expr);
    }

    #[test]
    fn comparison() {
        let source = "1.0 + 2.0 == 5.0 + 7.0;";
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr[0].to_string();

        assert_eq!(string_expr, "(== (+ 1 2) (+ 5 7))");
    }

    #[test]
    fn test_eq_with_paren() {
        let source = "1.0 == (2.0 + 2.0);";
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr[0].to_string();

        assert_eq!(string_expr, "(== 1 (group (+ 2 2)))");
    }
}
