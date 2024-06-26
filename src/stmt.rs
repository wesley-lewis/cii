use crate::expr::Expr;
use crate::scanner::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr },
    Var { name: Token, initializer: Expr },
    Block { statements: Vec<Box<Stmt>> },
    IfStmt { predicate: Expr, 
        then: Box<Stmt>, 
        els: Option<Box<Stmt>> 
    },
    WhileStmt {
        condition: Expr,
        body: Box<Stmt>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Box<Stmt>>,
    },
    // ForStmt {
    //     var_decl: Option<Box<Stmt>>,
    //     expr_stmt: Option<Box<Stmt>>,
    //
    //     condition: Option<Expr>,
    //     incrementer: Option<Expr>,
    //     body: Box<Stmt>,
}

impl Stmt {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        use Stmt::*;

        match self {
            Expression { expression } => expression.to_string(),
            Print { expression } => format!("(print {})", expression.to_string()),
            Var { name, initializer: _ } => format!("(var {})", name.lexeme),
            Block { statements } => {
                format!(
                    "(block {})", 
                    statements
                    .into_iter()
                    .map(|stmt| stmt.to_string())
                    .collect::<String>()
                )
            }
            IfStmt { predicate: _, then: _, els: _ } => todo!(),
            WhileStmt { condition: _condition, body: _body } => {
                todo!()
            },
            Function { name: _, params: _, body: _ } => {
                todo!()
            },
            // ForStmt { var_decl, condition, incrementer } => {
            // }
        }
    }
}
