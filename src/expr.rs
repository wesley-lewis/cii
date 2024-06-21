use crate::Token;
use crate::scanner;
use crate::environment::Environment;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub enum LiteralValue {
    Number(f32),
    StringValue(String),
    True,
    False,
    Nil,
    Callable {
        name: String,
        arity: usize,
        fun: Rc<dyn Fn(Rc<RefCell<Environment>>, &Vec<LiteralValue>) -> LiteralValue>,
    },
}

impl PartialEq for LiteralValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(x), Self::Number(y)) => x == y,
            (
                Self::Callable { name, arity, fun: _ }, 
                Self::Callable { name: name2, arity: arity2, fun: _ }
             ) => {
                name == name2 && arity == arity2
            },
            (Self::StringValue(x), Self::StringValue(y)) => x == y,
            (Self::True, Self::True) => true,
            (Self::False, Self::False) => true,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl std::fmt::Debug for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn unwrap_as_f32(literal: Option<scanner::LiteralValue>) -> f32 {
    match literal {
        Some(scanner::LiteralValue::FValue(x)) => x as f32,
        _ => panic!("could not unwrap as f32")
    }
}

fn unwrap_as_string(literal: Option<scanner::LiteralValue>) -> String {
    match literal {
        Some(scanner::LiteralValue::StringValue(s)) => s.clone(),
        _ => panic!("could not unwrap as string")
    }
}

impl LiteralValue {
    pub fn to_string(&self) -> String {
        match self {
            Self::Number(x) => x.to_string(),
            Self::StringValue(x) => format!("\"{}\"", &x),
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::Nil => "nil".to_string(),
            Self::Callable { name, arity, fun: _ } => format!("{name}/{arity}"),
        }
    }

    pub fn to_type(&self) -> &str {
        match self {
            Self::Number(_) => "Number",
            Self::StringValue(_) => "String",
            Self::True => "Boolean",
            Self::False => "Boolean",
            Self::Nil => "Nil",
            Self::Callable { name: _, arity: _, fun: _} => "Callable",
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
            Self::Nil => Self::True,
            Self::Callable { name: _, arity: _, fun: _ } => panic!("cannot use callable as truthy value"),
        }
    }

    pub fn is_truthy(&self) -> Self {
        match self {
            Self::Number(x) => {
                if *x == 0.0 { 
                    Self::False 
                } else { 
                    Self::True 
                }
            }, 
            Self::StringValue(s) => if s.len() == 0 { Self::False } else { Self::True },
            Self::True => Self::True,
            Self::False => Self::False,
            Self::Nil => Self::False,
            Self::Callable { name: _, arity: _, fun: _ } => panic!("cannot use callable as truthy value"),
        }
    }
}

#[derive(Clone)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    }
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Expr {
    pub fn to_string(&self) -> String {
        match self {
            Expr::Assign { name, value } => {
                format!("({} = {})", &name.lexeme, value.to_string())
            }
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
            Expr::Logical { left, operator, right } => format!("({} {} {})", operator.to_string(), left.to_string(), right.to_string()),
            Expr::Unary {operator, right } => {
                let operator_str = operator.lexeme.clone();
                let right_str = right.to_string();
                format!("({} {})", operator_str, right_str)
            },
            Expr::Variable { name } => format!("(var {})", name.lexeme),
            Expr::Call { callee, paren: _, arguments } => format!("({} {:?})", (*callee).to_string(), arguments),
        }
    }

    pub fn evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<LiteralValue, String> {
        use crate::scanner::TokenType::*;

        match self {
            Expr::Assign { name, value } => {
                let new_value = (*value).evaluate(environment.clone())?;
                let assign_success = environment.borrow_mut().assign(&name.lexeme, new_value.clone());
                if assign_success {
                    return Ok(new_value);
                }

                Err(format!("variable {} has not been declared", name.lexeme))
            },
            Expr::Variable{ name } => {
                match environment.borrow().get(name.lexeme.as_ref()) {
                    Some(value) => Ok(value.clone()),
                    None => Err(format!("Variable '{}' has not been declared", &name.lexeme))
                }
            },
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Logical { left, operator, right } => {
                match operator.token_type {
                    Or => {
                        let lhs_value = left.evaluate(environment.clone())?;
                        let lhs_true = lhs_value.is_truthy();
                        if lhs_true == LiteralValue::True {
                            return Ok(lhs_value);
                        }else {
                            right.evaluate(environment.clone())
                        }
                    },
                    And => {
                        let lhs_value = left.evaluate(environment.clone())?;
                        let lhs_true = lhs_value.is_truthy();
                        if lhs_true == LiteralValue::False {
                            return Ok(lhs_true);
                        }else {
                            right.evaluate(environment.clone())
                        }
                    },
                    ttype => Err(format!("Invalid token in logical expression: {}", ttype)),
                }
            }
            Expr::Grouping { expression } => expression.evaluate(environment),
            Expr::Unary { operator, right } => {
                let right = (*right).evaluate(environment)?;

                match (&right, operator.token_type) {
                    (LiteralValue::Number(x), Minus) => return Ok(LiteralValue::Number(-x)),
                    (_, Minus) => return Err(format!("minus not implemented for {}", right.to_type())),
                    (any, Bang) => Ok(any.is_falsy()),
                    (_, ttype) => Err(format!("{} is not a valid unary operator", ttype)),
                }
            },
            Expr::Binary { left, operator, right } => {
                let left: LiteralValue = left.evaluate(environment.clone())?;
                let right = right.evaluate(environment.clone())?;

                match (&left, operator.token_type, &right) {
                    (LiteralValue::Number(x),       Plus,           LiteralValue::Number(y)) => Ok(LiteralValue::Number(x + y)),
                    (LiteralValue::Number(x),       Minus,          LiteralValue::Number(y)) => Ok(LiteralValue::Number(x - y)),
                    (LiteralValue::Number(x),       Star,           LiteralValue::Number(y)) => Ok(LiteralValue::Number(x * y)),
                    (LiteralValue::Number(x),       Slash,          LiteralValue::Number(y)) => Ok(LiteralValue::Number(x / y)),
                    (LiteralValue::Number(x),       Greater,        LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x > y)),
                    (LiteralValue::Number(x),       GreaterEqual,   LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x >= y)),
                    (LiteralValue::Number(x),       Less,           LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x < y)),
                    (LiteralValue::Number(x),       LessEqual,      LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x <= y)),
                    (LiteralValue::Number(x),       BangEqual,      LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x != y)),
                    (LiteralValue::Number(x),       EqualEqual,     LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x == y)),

                    (LiteralValue::StringValue(_),  op,             LiteralValue::Number(_)) => Err(format!("'{}' is not defined for string and number", op)),
                    (LiteralValue::Number(_),       op,             LiteralValue::StringValue(_)) => Err(format!("'{}' is not defined for number and string", op)),

                    (LiteralValue::StringValue(s1), Plus,           LiteralValue::StringValue(s2)) => Ok(LiteralValue::StringValue(format!("{}{}", s1,s2))),
                    (LiteralValue::StringValue(s1), EqualEqual,     LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 == s2)),
                    (LiteralValue::StringValue(s1), BangEqual,      LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 != s2)),

                    (LiteralValue::StringValue(s1), Greater,        LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 > s2)),
                    (LiteralValue::StringValue(s1), GreaterEqual,   LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 >= s2)),
                    (LiteralValue::StringValue(s1), Less,           LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 < s2)),
                    (LiteralValue::StringValue(s1), LessEqual,      LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 <= s2)),
                    (x, ttype, y) => Err(format!("{} is not implemented for operands {} and {}", ttype, x.to_string(), y.to_string()))
                }
            },
            Expr::Call { callee, paren: _, arguments} => {
                // look up function definition in environment
                let callable = (*callee).evaluate(environment.clone())?;
                match callable {
                    LiteralValue::Callable { name, arity, fun } => {
                        // Do some checking (correct number of args?)
                        if arguments.len() != arity {
                            return Err(format!("Callable {} expected {} arguments but got {}", name, arity, arguments.len()));
                        }
                        // Evaluate arguments
                        let mut arg_vals = vec![];
                        for arg in arguments {
                            let val = arg.evaluate(environment.clone())?;
                            arg_vals.push(val);
                        }

                        // Apply to arguments
                        Ok(fun(environment.clone(), &arg_vals))
                    }
                    other => Err(format!("{} is not callable", other.to_type())),
                }
            }
        }
    }

    #[allow(warnings)]
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
