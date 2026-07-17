use std::{env, fs, io::{self, Write}, rc::Rc};

use rust_interpreter::{
    ast::visitor::Visitor,
    AstPrinter,
    ControlFlow,
    Interpreter,
    Parser,
    Resolver,
    Statement,
    scan,
};

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
            let ast: Vec<Statement> = parser.parse();

            // Print the AST using the visit method
            let mut printer = AstPrinter::new();
            printer.print_statements(ast.into_iter().map(|stmt| Rc::new(stmt)).collect());
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
            let mut interpreter = Interpreter::new(Resolver::new());
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
            let statements = parser.parse();
            let statement_refs: Vec<Rc<Statement>> = statements.into_iter().map(|stmt| Rc::new(stmt)).collect();

            let mut resolver = Resolver::new();
            resolver.resolve_statements(statement_refs.clone());

            // Create an interpreter and execute the statements
            let mut interpreter = Interpreter::new(resolver);

            interpreter.interpret(statement_refs);
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
