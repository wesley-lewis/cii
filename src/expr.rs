use crate::Token;
use crate::scanner;

#[derive(Clone, PartialEq, Debug)]
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

    pub fn to_type(&self) -> &str {
        match self {
            Self::Number(_) => "Number",
            Self::StringValue(_) => "String",
            Self::True => "Boolean",
            Self::False => "Boolean",
            Self::Nil => "Nil",
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

    pub fn from_bool(b: bool) -> Self {
        if b {
            Self::True
        }else {
            Self::False
        }
    }

    pub fn is_falsy(&self) -> Self {
        match self {
            Self::Number(x) => if *x == 0.0 { Self::True } else { Self::False }, 
            Self::StringValue(s) => if s.len() == 0 { Self::True } else { Self::False },
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Nil => Self::True
        }
    }
}

#[derive(Debug, Clone)]
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

    pub fn evaluate(&self) -> Result<LiteralValue, String> {
        use crate::scanner::TokenType::*;

        match self {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Grouping { expression } => expression.evaluate(),
            Expr::Unary { operator, right } => {
                let right = (*right).evaluate()?;

                match (&right, operator.token_type) {
                    (LiteralValue::Number(x), Minus) => return Ok(LiteralValue::Number(-x)),
                    (_, Minus) => return Err(format!("minus not implemented for {}", right.to_type())),
                    (any, Bang) => Ok(any.is_falsy()),
                    (_, ttype) => Err(format!("{} is not a valid unary operator", ttype)),
                }
            },
            Expr::Binary { left, operator, right } => {
                let left: LiteralValue = left.evaluate()?;
                let right = right.evaluate()?;

                match (&left, operator.token_type, &right) {
                    (LiteralValue::Number(x), Plus, LiteralValue::Number(y)) =>  Ok(LiteralValue::Number(x + y)),
                    (LiteralValue::Number(x), Minus, LiteralValue::Number(y)) => Ok(LiteralValue::Number(x - y)),
                    (LiteralValue::Number(x), Star, LiteralValue::Number(y)) =>  Ok(LiteralValue::Number(x * y)),
                    (LiteralValue::Number(x), Slash, LiteralValue::Number(y)) => Ok(LiteralValue::Number(x / y)),
                    (LiteralValue::Number(x), Greater, LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x > y)),
                    (LiteralValue::Number(x), GreaterEqual, LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x >= y)),
                    (LiteralValue::Number(x), Less, LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x < y)),
                    (LiteralValue::Number(x), LessEqual, LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x <= y)),
                    // (LiteralValue::Number(x), BangEqual, LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x != y)),
                    // (LiteralValue::Number(x), EqualEqual, LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x == y)),

                    (LiteralValue::StringValue(_), op, LiteralValue::Number(_)) => Err(format!("'{}' is not defined for string and number", op)),
                    (LiteralValue::Number(_), op, LiteralValue::StringValue(_)) => Err(format!("'{}' is not defined for number and string", op)),

                    (LiteralValue::StringValue(s1), Plus, LiteralValue::StringValue(s2)) => Ok(LiteralValue::StringValue(format!("{}{}", s1,s2))),
                    // (LiteralValue::StringValue(s1), EqualEqual, LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 == s2)),
                    // (LiteralValue::StringValue(s1), BangEqual, LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 != s2)),

                    (LiteralValue::StringValue(s1), Greater, LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 > s2)),
                    (LiteralValue::StringValue(s1), GreaterEqual, LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 >= s2)),
                    (LiteralValue::StringValue(s1), Less, LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 < s2)),
                    (LiteralValue::StringValue(s1), LessEqual, LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 <= s2)),
                    (x, ttype, y) => Err(format!("{} is not implemented for operands {:?} and {:?}", ttype, x, y))
                }
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
