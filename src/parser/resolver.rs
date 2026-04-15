use std::collections::HashMap;
use std::cell::RefCell;
use crate::Interpreter;
use crate::Statement;
use crate::Expr;
use crate::Token;
use crate::ParseError;
use crate::ast::Depth;
use crate::ast::VisitorMutable;

/// Type alias for a scope lookup table (maps variable names to defined status)
pub type Lookup = RefCell<HashMap<String, bool>>;
pub type Output = Result<(), ParseError>;

/// Enum to track the type of function currently being resolved
#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver {
    scopes: Vec<Lookup>,
    current_function: FunctionType,
}

impl Resolver {
    /// Create a new Resolver with a reference to the interpreter
    pub fn new() -> Self {
        Resolver {
            scopes: Vec::new(),
            current_function: FunctionType::None,
        }
    }

    /// Create and return a parse error with a message at a given token
    fn error(token: &Token, message: &str) -> Output {
        let message = format!("At '{}': {}", token.lexeme, message);
        return Err(ParseError { line: token.line, message: message.to_string() })
    }

    /// Resolve a list of statements by resolving each statement in order
    pub fn resolve_statements(&mut self, statements: &mut Vec<Statement>) {
        // Resolve each statement in the list
        for statement in statements {
            if let Err(parse_error) = self.visit_statement(statement) {
                eprintln!("{}", parse_error);
                std::process::exit(65);
            }
        }
    }

    /// Resolve a local variable by determining its scope depth
    fn resolve_local(&mut self, name: &Token, depth: &mut Depth) -> Output {
        // Look for the variable in each scope, starting from the innermost
        for (index, scope) in self.scopes.iter().rev().enumerate() {
            // If found, inform the interpreter of the variable's depth
            if self.is_declared(&name.lexeme, scope)? {
                *depth = Depth::Resolved(self.scopes.len() - (index + 1));
                dbg!("Resolved variable '{}' at depth {}", &name.lexeme, *depth);
            }
        }

        Ok(())
    }

    fn begin_scope(&mut self) -> Output {
        // Push a new, empty scope onto the stack
        self.scopes.push(Lookup::new(HashMap::new()));

        Ok(())
    }

    fn end_scope(&mut self) -> Output {
        // Pop the top scope off the stack
        self.scopes.pop();

        Ok(())
    }

    /// Get the top scope from the stack
    fn get_top(&self) -> Result<&Lookup, ParseError> {
        if let Some(top) = self.scopes.last() {
            return Ok(top);
        }
        return Err(ParseError { line: 0, message: "Failed to read scope".to_string() })
    }

    /// Get the value associated with a variable name in a given scope (None if not found)
    fn get(&self, name: &Token, scope: &Lookup) -> Result<Option<bool>, ParseError> {
        return Ok(scope.borrow_mut().get(&name.lexeme).cloned());
    }

    /// Declare a variable in the current scope (with false in the map for "not yet defined")
    fn declare(&mut self, name: &Token) -> Output {
        // If no scopes, we're in global scope, so nothing to do
        if self.scopes.is_empty() { return Ok(()) }

        // Check if variable with this name already declared in this scope
        else if self.is_declared(&name.lexeme, self.get_top()?)? {
            return Self::error(name, "Variable with this name already declared in this scope");
        }

        let current_scope = self.scopes.last().unwrap();
        current_scope.borrow_mut().insert(name.to_string(), false);

        Ok(())
    }

    /// Check if a variable name is declared in a given scope
    fn is_declared(&self, name: &String, scope: &Lookup) -> Result<bool, ParseError> {
        return Ok(scope.borrow_mut().contains_key(name));
    }

    /// Define a variable in the current scope (with true in the map for "defined")
    fn define(&mut self, name: &Token) -> Output {
        if self.scopes.is_empty() { return Ok(()) }

        let current_scope = self.get_top()?;
        current_scope.borrow_mut().insert(name.lexeme.to_string(), true);

        Ok(())
    }

    /// Resolve a function by creating a new scope for its parameters and body
    fn resolve_function(&mut self, params: &mut Vec<Token>, body: &mut Vec<Statement>, function_type: FunctionType) -> Output {
        // Keep track of the enclosing function type
        let enclosing_function = self.current_function;
        self.current_function = function_type;
        
        // Begin a new scope for the function body
        self.begin_scope()?;

        // Bind variables for each of the parameters
        for param in params {
            self.declare(param)?;
            self.define(param)?;
        }
        
        // Resolve the function body in its own scope
        self.visit_block_statement(body)?;
        
        // End the function scope
        self.end_scope()?;

        // Restore the previous function type
        self.current_function = enclosing_function;

        Ok(())
    }
}

impl VisitorMutable<Output> for Resolver {
    /// Resolve a block statement by creating a new scope for its statements
    fn visit_block_statement(&mut self, statements: &mut [Statement]) -> Output {
        self.begin_scope()?;

        // Resolve each statement in the block in the new scope
        for statement in statements {
            self.visit_statement(statement)?;
        }

        self.end_scope()?;

        Ok(())
    }

    /// Resolve a variable declaration statement by declaring, resolving initializer, and defining the variable
    fn visit_var_statement(&mut self, name: &mut Token, initializer: &mut Option<Expr>) -> Output {
        // Exists, but undefined
        self.declare(name)?;

        // Resolve the initializer expression if it exists
        if initializer.is_some() {
            self.visit_expression(initializer.as_mut().unwrap())?;
        }

        self.define(name)?;
        Ok(())
    }

    /// Resolve an if statement by resolving its condition and branches
    fn visit_if_statement(&mut self, condition: &mut Expr, then_branch: &mut Statement, else_branch: &mut Option<Box<Statement>>) -> Output {
        self.visit_expression(condition)?;
        self.visit_statement(then_branch)?;
        if else_branch.is_some() {
            self.visit_statement(else_branch.as_mut().unwrap())?;
        }

        Ok(())
    }

    /// Resolve a print statement by resolving its expression
    fn visit_print_statement(&mut self, expression: &mut Expr) -> Output {
        self.visit_expression(expression)?;

        Ok(())
    }

    /// Resolve a return statement by resolving its return value (if any)
    fn visit_return_statement(&mut self, keyword: &mut Token, value: &mut Option<Expr>, ) -> Output {
        // Error if return used outside of function
        if self.current_function == FunctionType::None {
            return Self::error(keyword, "Can't return from top-level code");
        }
        
        if value.is_some() {
            self.visit_expression(value.as_mut().unwrap())?;
        }

        Ok(())
    }

    /// Resolve a while statement by resolving its condition and body
    fn visit_while_statement(&mut self, condition: &mut Expr, body: &mut Statement) -> Output {
        self.visit_expression(condition)?;
        self.visit_statement(body)?;

        return Ok(())
    }

    /// Resolve a function statement by declaring its name and resolving its parameters and body
    fn visit_function_statement(&mut self, statement: &mut Statement) -> Output {
        // Declare the function name
        if let Statement::Function { name, params, body, .. } = statement {
             self.declare(name)?;
             self.define(name)?;

             self.resolve_function(params, body, FunctionType::Function)?;
        }
        Ok(())
    }

    /// Resolve an assignment expression ("a" = "b") by resolving the assigned value and the variable being assigned
    fn visit_assign(&mut self, name: &mut Token, value: &mut Expr, depth: &mut Depth) -> Output {
        // Resolve assigned value in case it contains references to other variables
        self.visit_expression(value)?;
        // Resolve the variable that is being assigned
        self.resolve_local(name, depth)?;

        Ok(())
    }

    /// Resolve a variable expression (like "my_variable") by determining its scope depth
    fn visit_variable(&mut self, name: &mut Token, depth: &mut Depth) -> Output {
        // (Check if scopes are empty to avoid error) If variable used inside its own declaration, error
        if !self.scopes.is_empty() && self.get(&name, self.get_top()?)? == Some(false) {
            return Self::error(&name, "Can't read local variable in its own initializer" );
        }

        self.resolve_local(name, depth)?;
        return Ok(());
    }

    /// Resolve a binary expression by resolving its left and right operands
    fn visit_binary(&mut self, left: &mut Expr, _operator: &mut Token, right: &mut Expr) -> Output {
        self.visit_expression(left)?;
        self.visit_expression(right)?;

        Ok(())
    }

    /// Resolve a call expression by resolving its callee and argument expressions
    fn visit_call(&mut self, callee: &mut Expr, _paren: &mut Token, arguments: &mut Vec<Expr>) -> Output {
        // Resolve the callee expression
        self.visit_expression(callee)?;

        // Resolve each argument expression
        for argument in arguments {
            self.visit_expression(argument)?;
        }

        Ok(())
    }

    /// Resolve a grouping expression by resolving the inner expression
    fn visit_grouping(&mut self, expression: &mut Expr) -> Output {
        self.visit_expression(expression)?;

        Ok(())
    }

    /// Resolve a logical expression by resolving its left and right operands
    fn visit_logical_and(&mut self, left: &mut Expr, right: &mut Expr) -> Output {
        self.visit_expression(left)?;
        self.visit_expression(right)?;

        Ok(())
    }

    fn visit_logical_or(&mut self, left: &mut Expr, right: &mut Expr) -> Output {
        self.visit_expression(left)?;
        self.visit_expression(right)?;

        Ok(())
    }

    /// Resolve a unary expression by resolving its operand
    fn visit_unary(&mut self, _operator: &mut Token, right: &mut Expr) -> Output {
        self.visit_expression(right)?;

        Ok(())
    }

    fn visit_get(&mut self, object: &mut Expr, _name: &mut Token) -> Output {
        self.visit_expression(object)?;

        Ok(())
    }

    fn visit_lambda(&mut self, params: &mut Vec<Token>, body: &mut Vec<Statement>) -> Output {
        self.resolve_function(params, body, FunctionType::Function)?;

        Ok(())
    }

    fn visit_literal(&mut self, _value: &mut Token) -> Output {
        Ok(())
    }

    fn visit_class_statement(&mut self, name: &mut Token, methods: &mut [Statement]) -> Output {
        // Declare the class name
        self.declare(name)?;
        self.define(name)?;

        Ok(())
    }

    fn visit_expression_statement(&mut self, expression: &mut Expr) -> Output {
        self.visit_expression(expression)
    }
}