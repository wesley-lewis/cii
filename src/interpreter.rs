use crate::environment::Environment;
use crate::stmt::Stmt; 
use crate::scanner::Token;
use crate::expr::LiteralValue;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Interpreter {
    // globals: Environment,
    environment: Rc<RefCell<Environment>>,
}

fn clock_impl(_env: Rc<RefCell<Environment>>, _args: &Vec<LiteralValue>) -> LiteralValue {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // LiteralValue::Number(now as f32)
    LiteralValue::StringValue(now.to_string())
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();
        globals.define("clock".to_string(), LiteralValue::Callable { 
            name: "clock".to_string(), 
            arity: 0,
            fun: Rc::new(clock_impl),
        });
        Self {
            // globals,
            // environment: Rc::new(RefCell::new(Environment::new())),
            environment: Rc::new(RefCell::new(globals)),
        }
    }

    fn for_closure(parent: Rc<RefCell<Environment>>) -> Self {
        let environment = Rc::new(RefCell::new(Environment::new()));
        environment.borrow_mut().enclosing = Some(parent);

        Self {
            environment
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
                    let stmts = statements.into_iter().map(|b| b.as_ref()).collect();
                    self.interpret(stmts)?;
                    self.environment = old_environment;
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
                },
                Stmt::Function { name, params, body } => {
                    // Function decl
                    let arity = params.len();
                    // Function impl:
                    // Bind list of input values to params
                    // Add those bindings to the environment used to execute body
                    // Then execute body

                    let params: Vec<Token> = params.iter().map(|t| (*t).clone()).collect();

                    let body: Vec<Box<Stmt>> = body.iter().map(|b| (*b).clone()).collect();
                    
                    let name_clone = name.clone();

                    // TODO: make a struct that contains data for evaluation
                    // and which implements Fn
                    let fun_impl = move |parent_env, args: &Vec<LiteralValue>| {
                    let mut closure_interpreter = Interpreter::for_closure(parent_env);
                        for (i, arg) in args.iter().enumerate() {
                            closure_interpreter.environment
                                .borrow_mut()
                                .define(params[i].lexeme.clone(), (*arg).clone()
                            );
                        }

                        for i in 0..(body.len() - 1) {
                            closure_interpreter
                                .interpret(vec![&body[i]])
                                .expect(
                                    &format!("evaluating failed inside {}", 
                                        name_clone.lexeme.clone())
                                );
                        }

                        let value;
                        match &body[body.len() - 1].as_ref() {
                            &Stmt::Expression { expression } => {
                                value = expression
                                            .evaluate(closure_interpreter.environment)
                                            .unwrap();
                            },
                            _ => todo!()
                        }

                        value
                    }; // closure end

                    let callable = LiteralValue::Callable {
                        name: name.lexeme.clone(),
                        arity,
                        fun: Rc::new(fun_impl),
                    };

                    
                    self.environment.borrow_mut().define(name.lexeme.clone(), callable);
                }
            };
        }

        Ok(())
    }
}
