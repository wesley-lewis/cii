mod scanner;
mod expr;
mod parser;
use parser::Parser;

use crate::scanner::*;

use std::env;
use std::io::Write;
use std::fs;
use std::io;
use std::process::exit;

fn run_file(path: &str) -> Result<(), String> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            return run(&contents);
        },
        Err(e) => {
            return Err(e.to_string());
        }
    }
}

fn run(contents: &str) -> Result<(), String> {
    let mut scanner = Scanner::new(contents);
    let tokens = scanner.scan_tokens()?;
    
    let mut parser = Parser::new(tokens);
    let parsed_expr = parser.parse()?;
    let result = parsed_expr.evaluate()?;
    println!("{}", result.to_string());

    Ok(())
}

fn run_prompt() -> Result<(), String>{
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
        print!("ECHO: ");
        match run(&buffer) {
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
// part 11 done
