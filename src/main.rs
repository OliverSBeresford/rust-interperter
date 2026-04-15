use std::env;
use std::fs;
use std::io::{self, Write};
use rust_interpreter::parser::Resolver;
use rust_interpreter::ast::visitor::Visitor;

use rust_interpreter::{AstPrinter, ControlFlow, Interpreter, Parser, scan};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    // The command to execute: tokenize, parse, evaluate, run, dbg
    let command = &args[1];
    let filename = &args[2];

    // Read the file contents into a string
    let file_contents = match fs::read_to_string(filename) {
        Ok(file_string) => file_string,
        Err(error_message) => {
            eprintln!("Failed to read file {}: {}", filename, error_message);
            std::process::exit(1);
        }
    };

    match command.as_str() {
        // Tokenize the input file and print the tokens
        "tokenize" => {
            if file_contents.is_empty() {
                println!("EOF  null");
                return;
            }

            let tokens = scan(&file_contents);

            // Tokenize the input and print the tokens
            print!("{}", tokens); 
        }
        // Parse the input file and print the AST
        "parse" => {
            // Get tokens from the scanner
            let tokens = scan(&file_contents);
            
            // Create a parser and parse the tokens into an AST
            let mut parser = Parser::new(tokens.tokens);
            let expression = parser.expression();

            // Print the AST using the visit method
            match expression {
                Ok(expr) => {
                    let mut printer = AstPrinter;
                    printer.print_expression(&expr);
                }
                Err(error) => {
                    eprintln!("{}", error);
                    std::process::exit(65);
                }
            }
        }
        // Evaluate the input file and print the result
        "evaluate" => {
            // Get tokens from the scanner
            let tokens = scan(&file_contents);
            
            // Create a parser and parse the tokens into an AST
            let mut parser = Parser::new(tokens.tokens);
            let expression = parser.expression().unwrap_or_else(|error| {
                eprintln!("{}", error);
                std::process::exit(65);
            });

            // Create an interpreter and evaluate the expression
            let mut interpreter = Interpreter::new();
            let result = interpreter.visit_expression(&expression).unwrap_or_else(|control_flow| {
                if let ControlFlow::RuntimeError(runtime_error) = control_flow {
                    eprintln!("{}", runtime_error);
                    std::process::exit(70);
                }
                std::process::exit(70);
            });
            
            // Print the result of the evaluation
            println!("{}", result);
        }
        // Run the input file as a series of statements
        "run" => {
            // Get tokens from the scanner
            let tokens = scan(&file_contents);
            
            // Create a parser and parse the tokens into statements
            let mut parser = Parser::new(tokens.tokens);
            let mut statements = parser.parse();

            // Create an interpreter and execute the statements
            let mut interpreter = Interpreter::new();

            let mut resolver = Resolver::new(&mut interpreter);
            resolver.resolve_statements(&mut statements);

            interpreter.interpret(&statements);
        }
        // Debug: Print the tokens and parsed statements AST
        "dbg" => {
            // Get tokens from the scanner
            let tokens = scan(&file_contents);
            println!("Tokens:\n{}\n", tokens);
            
            // Create a parser and parse the tokens into statements
            let mut parser = Parser::new(tokens.tokens);
            let statements = parser.parse();

            // Print the AST of the statements
            dbg!("Parsed Statements AST:", &statements);
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}
