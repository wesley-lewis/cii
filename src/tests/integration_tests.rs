use std::process::{Command, Stdio};

use crate::{interpreter::Interpreter, parser::Parser, Scanner};

pub fn run_and_capture(command: &mut Command) -> std::process::Output {
    command.output().unwrap()
}

#[test]
fn interpret_block() {
    let output = Command::new("./target/debug/cii").args(vec!["./src/tests/cases/block.lox"]).output().unwrap();

    let output = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = output.split("\n").collect();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "3");
    assert_eq!(lines[1], "3");
}

#[test]
fn interpret_while() {
    let output = Command::new("./target/debug/cii").args(vec!["./src/tests/cases/while.lox"]).output().unwrap();

    let output = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = output.split("\n").collect();

    assert_eq!(lines[0], "1");
}

#[test]
fn interpret_while_math() {
    let output = Command::new("./target/debug/cii").args(vec!["./src/tests/cases/while_math.lox"]).output().unwrap();

    let output = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = output.split("\n").collect();
    assert_eq!(lines.len(), 11);
    assert_eq!(lines[0], "10");
    assert_eq!(lines[1], "90");
    assert_eq!(lines[2], "720");
    assert_eq!(lines[3], "5040");
    assert_eq!(lines[4], "30240");
    assert_eq!(lines[5], "151200");
    assert_eq!(lines[6], "604800");
    assert_eq!(lines[7], "1814400");
    assert_eq!(lines[8], "3628800");
}

#[test]
fn test_bug() {
    let source = std::fs::read_to_string("src/tests/cases/while.lox").unwrap();
    let mut scanner = Scanner::new(&source);
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let mut interpreter = Interpreter::new();
    interpreter.interpret(statements.iter().map(|stmt| stmt).collect()).unwrap();
}
