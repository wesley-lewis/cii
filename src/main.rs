mod scanner;
mod expr;
mod parser;
mod interpreter;
mod stmt;
mod environment;

use parser::Parser;

use crate::scanner::*;
use crate::interpreter::Interpreter;

use std::env;
use std::io::Write;
use std::fs;
use std::io;
use std::process::exit;

fn run_file(path: &str) -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    match fs::read_to_string(path) {
        Ok(contents) => {
            return run(&mut interpreter, &contents);
        },
        Err(e) => {
            return Err(e.to_string());
        }
    }
}

fn run(interpreter: &mut Interpreter, contents: &str) -> Result<(), String> {
    let mut scanner = Scanner::new(contents);
    let tokens = scanner.scan_tokens()?;
    
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse()?;
    interpreter.interpret(stmts)?;

    Ok(())
}

fn run_prompt() -> Result<(), String>{
    let mut interpreter: Interpreter = Interpreter::new();
    loop {
        print!("> ");
        match io::stdout().flush() { // need to flush to stdout, else it doesn't print to the terminal
            Ok(_) => {},
            Err(_) => return Err("couldn't flush stdout".to_string()),
        }

        let mut buffer = String::new();
        let stdin = io::stdin();

        match stdin.read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    return Ok(());
                }
            },
            Err(e) => {
                return Err(e.to_string());
            }
        }
        print!("ECHO: {}", &buffer);
        match run(&mut interpreter, &buffer) {
            Ok(_) => {},
            Err(e) => eprintln!("ERROR: {}", e),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: jlox [script]");
        exit(64);
    }else if args.len() == 2 {
        match run_file(&args[1]) {
            Ok(_) => exit(0),
            Err(e) => println!("ERROR: {}", e),
        }
    }else {
        match run_prompt() {
            Ok(_) => exit(0),
            Err(msg) => {
                eprintln!("ERROR: {}", msg);
            }
        }
    }
}

// 1) basic structure of project parser setup 
// 2) structure of tokens and token type are defined
// 3) started to scan tokens and manage errors
// 4) tokenizing one and two chars
// 5) tokenized string literals
// 6) tokenized numbers into floats(64 bits) and integers(64 bits)
// 7) tokenized keywords and identifiers, defined all language keywords
// 8) created AST and printed it out
// 9) simple mathematical parser done. Still need to test it out though
// 10) tested the above and its parsing expressions and conditionals
// 11) evaluating expressions, numbers and string comparison evaluated
// 12) interpreting arithmetic, basic arithmetic calculator is ready!
// 13) print expression working
// 14) declaring variables and storing them into the environment also evaluating expressions using
//     variable
// 15) Variable assignment and reassignment
// 16) handling lifetimes is difficult :)
// 17) scaffolding for `Scope` ready
// part 17 done
