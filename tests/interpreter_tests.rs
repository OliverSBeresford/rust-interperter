use rust_interpreter::{Interpreter, Parser, Value, scan};
use std::rc::Rc;
use rust_interpreter::runtime::{Callable, EnvRef, Environment, Function};
use rust_interpreter::Expr;
use rust_interpreter::ast::{Statement, Visitor};
use rust_interpreter::Resolver;

fn parse_expr(input: &str) -> (Interpreter, Expr) {
    let tokens = scan(input);
    let mut parser = Parser::new(tokens.tokens);
    let expr = parser.expression().unwrap_or_else(|e| panic!("parse error: {}", e));
    (Interpreter::new(), expr)
}

fn parse_stmts(input: &str) -> (Interpreter, Vec<Statement>) {
    let tokens = scan(input);
    let mut parser = Parser::new(tokens.tokens);
    let mut statements = parser.parse();
    let interpreter = Interpreter::new();
    let mut resolver = Resolver::new();
    resolver.resolve_statements(&mut statements);
    (interpreter, statements)
}

#[test]
fn evaluate_addition() {
    let (mut interpreter, expr) = parse_expr("1 + 2");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Integer(n) => assert_eq!(n, 3),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_unary_minus() {
    let (mut interpreter, expr) = parse_expr("-5");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Integer(n) => assert_eq!(n, -5),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_logic_not_truthiness() {
    let (mut interpreter, expr) = parse_expr("!123");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Bool(b) => assert_eq!(b, false),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_variable_lookup() {
    let (mut interpreter, expr) = parse_expr("a");
    // define variable in environment
    interpreter.environment.borrow_mut().define("a".to_string(), Value::Integer(42));
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Integer(n) => assert_eq!(n, 42),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn function_call_returns_sum() {
    // Parse a function declaration
    let (mut interpreter, statements) = parse_stmts(
        "
        fun add(x, y) {
            return x + y;
        }
        ",
    );
    // Make sure we have only one statement which is the function declaration
    assert!(statements.len() == 1, "expected one statement");
    let stmt = statements.into_iter().next().expect("one statement expected");
    
    // bind in current environment
    let env: EnvRef = Environment::new(None);
    interpreter.environment = env.clone();

    // Build function from statement
    let func = Function::from_statement(&stmt, env.clone()).unwrap_or_else(|_| panic!("function build error"));
    
    // Call the function with args
    let result = Rc::new(func);
    let result = result.call(&mut interpreter, vec![Value::Float(2.0), Value::Float(3.0)]);
    match result {
        Ok(value) => assert!(matches!(value, Value::Float(n) if n == 5.0)),
        Err(control_flow) => {
            panic!("unexpected control flow: {:?}", control_flow);
        }}
}

#[test]
fn evaluate_string_concatenation() {
    let (mut interpreter, expr) = parse_expr("\"hello\" + \" world\"");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Str(s) => assert_eq!(s, "hello world"),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_multiplication_and_division() {
    let (mut interpreter, expr) = parse_expr("6 * 7");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Integer(n) => assert_eq!(n, 42),
        other => panic!("unexpected value: {:?}", other),
    }
    
    let (mut interpreter, expr) = parse_expr("20 / 4");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Float(n) => assert_eq!(n, 5.0),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_comparison_operators() {
    let (mut interpreter, expr) = parse_expr("5 > 3");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Bool(b) => assert_eq!(b, true),
        other => panic!("unexpected value: {:?}", other),
    }
    
    let (mut interpreter, expr) = parse_expr("2 <= 10");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Bool(b) => assert_eq!(b, true),
        other => panic!("unexpected value: {:?}", other),
    }
    
    let (mut interpreter, expr) = parse_expr("7 == 7");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Bool(b) => assert_eq!(b, true),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_logical_operators() {
    let (mut interpreter, expr) = parse_expr("true and false");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Bool(b) => assert_eq!(b, false),
        other => panic!("unexpected value: {:?}", other),
    }
    
    let (mut interpreter, expr) = parse_expr("true or false");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Bool(b) => assert_eq!(b, true),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_grouped_expressions() {
    let (mut interpreter, expr) = parse_expr("(1 + 2) * 3");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Integer(n) => assert_eq!(n, 9),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_nil_value() {
    let (mut interpreter, expr) = parse_expr("nil");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Nil => {},
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_subtraction() {
    let (mut interpreter, expr) = parse_expr("10 - 3");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Integer(n) => assert_eq!(n, 7),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_nested_arithmetic() {
    let (mut interpreter, expr) = parse_expr("2 * 3 + 4 * 5");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Integer(n) => assert_eq!(n, 26),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_inequality() {
    let (mut interpreter, expr) = parse_expr("5 != 3");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Bool(b) => assert_eq!(b, true),
        other => panic!("unexpected value: {:?}", other),
    }
    
    let (mut interpreter, expr) = parse_expr("7 != 7");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Bool(b) => assert_eq!(b, false),
        other => panic!("unexpected value: {:?}", other),
    }
}

#[test]
fn evaluate_boolean_literals() {
    let (mut interpreter, expr) = parse_expr("true");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Bool(b) => assert_eq!(b, true),
        other => panic!("unexpected value: {:?}", other),
    }
    
    let (mut interpreter, expr) = parse_expr("false");
    let v = interpreter.visit_expression(&expr).unwrap_or_else(|_| panic!("eval error"));
    match v {
        Value::Bool(b) => assert_eq!(b, false),
        other => panic!("unexpected value: {:?}", other),
    }
}
