#![allow(warnings)]
use crate::Token;
use crate::scanner;

#[derive(Clone)]
pub enum LiteralValue {
    Number(f32),
    StringValue(String),
    True,
    False,
    Nil,
}

fn unwrap_as_f32(literal: Option<scanner::LiteralValue>) -> f32 {
    match literal {
        Some(scanner::LiteralValue::IntValue(x)) => x as f32,
        Some(scanner::LiteralValue::FValue(x)) => x as f32,
        _ => panic!("could not unwrap as f32")
    }
}

fn unwrap_as_string(literal: Option<scanner::LiteralValue>) -> String {
    match literal {
        Some(scanner::LiteralValue::StringValue(s)) => s.clone(),
        Some(scanner::LiteralValue::IdentifierValue(s)) => s.clone(),
        _ => panic!("could not unwrap as string")
    }
}

impl LiteralValue {
    pub fn to_string(&self) -> String {
        match self {
            Self::Number(x) => x.to_string(),
            Self::StringValue(x) => x.clone(),
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::Nil => "nil".to_string(),
        }
    }

    pub fn from_token(token: Token) -> LiteralValue {
        use crate::scanner::TokenType::*;
        match token.token_type {
            Number => Self::Number(unwrap_as_f32(token.literal)),
            StringLit => Self::StringValue(unwrap_as_string(token.literal)),
            False => Self::False,
            True => Self::True,
            Nil => Self::Nil,
            _ => panic!("could not create LiteralValue from {:?}", token)
        }
    }
}

#[derive(Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl Expr {
    pub fn to_string(&self) -> String {
        match self {
            Expr::Binary { left, operator, right } => {
                format!("({} {} {})", 
                    &operator.lexeme, 
                    left.to_string(), 
                    right.to_string()
                )
            },
            Expr::Grouping { expression } => format!("(group {})", expression.to_string()),
            Expr::Literal { value } => {
                format!("{}", value.to_string())
            },
            Expr::Unary {operator, right } => {
                let operator_str = operator.lexeme.clone();
                let right_str = right.to_string();
                format!("({} {})", operator_str, right_str)
            }
        }
    }

    pub fn print(&self) {
        println!("{}", self.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TokenType;
    use crate::Token;

    #[test]
    fn pretty_print_ast() {
        let minus_token = Token::new( 
            TokenType::Minus,
            "-".to_string(),
            None,
            1,
        );
        let onetwothree = Expr::Literal{ value: LiteralValue::Number(123.0)};
        let multi = Token::new(TokenType::Star, "*".to_string(), None, 1);
        let group = Expr::Grouping {
            expression: Box::new(Expr::Literal{ value: LiteralValue::Number(45.67)}),
        };
        let ast = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: minus_token,
                right: Box::new(onetwothree),
            }),
            operator: multi,
            right: Box::new(group),
        };
        let result = ast.to_string();
        assert_eq!(result, "(* (- 123) (group 45.67))");
    }
}
