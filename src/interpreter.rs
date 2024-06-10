use crate::environment::Environment;
use crate::stmt::Stmt;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<(), String> {
        use crate::expr::LiteralValue;
        for stmt in stmts {
            match stmt {
                Stmt::Expression { expression } => {
                    expression.evaluate(
                        self.environment.clone()
                    )?;
                },
                Stmt::Print { expression } => {
                    let value = expression.evaluate(
                        self.environment.clone()
                    )?;
                    println!("{}", value.to_string());
                },
                Stmt::Var { name, initializer } => {
                    let value = initializer.evaluate(
                        self.environment.clone()
                    )?;

                    self.environment.borrow_mut()
                        .define(name.lexeme.to_string(), value);

                },
                Stmt::Block { statements } => {
                    let mut new_environment = Environment::new();
                    new_environment.enclosing = Some(self.environment.clone());

                    let old_environment = self.environment.clone();
                    self.environment = Rc::new(RefCell::new(new_environment));
                    let block_result = self.interpret(statements.into_iter().map(|b| b.as_ref()).collect());
                    self.environment = old_environment;

                    return block_result;
                },
                Stmt::IfStmt { predicate, then, els } => {

                    let truth_value = predicate.evaluate(
                        self.environment.clone()
                    )?;
                    if truth_value.is_truthy() == LiteralValue::True {
                        self.interpret(vec![then.as_ref()])?
                    }else if let Some(els_stmt) = els {
                        self.interpret(vec![els_stmt.as_ref()])?
                    }
                }
                Stmt::WhileStmt { condition, body } => {
                    let mut flag = condition.evaluate(self.environment.clone())?;
                    while flag.is_truthy() == LiteralValue::True {
                        // TODO: do we to interpret statements in their own block
                        self.interpret(vec![body.as_ref()])?;
                        flag = condition.evaluate(
                            self.environment.clone()
                        )?;
                    }
                }
            };
        }

        Ok(())
    }
}
