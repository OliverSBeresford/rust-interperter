use std::fmt;
use std::rc::Rc;
use crate::ast::{Depth, Expr, Statement, Visitor};
use crate::lexer::token::{Literal, Token, TokenType};
use crate::runtime::clock::Clock;
use crate::runtime::control_flow::ControlFlow;
use crate::runtime::environment::{EnvRef, Environment};
use crate::runtime::function::Function;
use crate::runtime::callable::Callable;
use crate::runtime::runtime_error::RuntimeError;
use crate::runtime::value::Value;
use crate::runtime::class::Class;

pub type InterpreterResult<T> = Result<T, ControlFlow>;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = match self {
            Value::Integer(i) => format!("{}", i),
            Value::Float(n) => {
                // If the value is an integer (no fractional part) print one decimal place
                // Otherwise print the float normally.
                format!("{}", n)
            }
            Value::Str(s) => s.clone(),
            Value::Bool(b) => format!("{}", b),
            Value::Nil => "nil".to_string(),
            Value::Callable(func) => format!("{}", func.to_string()),
            Value::Instance(instance) => format!("<instance of {}>", instance.class.name()),
        };
        write!(f, "{}", out)
    }
}

pub struct Interpreter {
    pub globals: EnvRef,
    pub environment: EnvRef,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new(None);
        let interpreter = Interpreter {
            globals: globals.clone(),
            environment: globals.clone(),
        };
        // Define native functions in the global environment
        interpreter
            .globals
            .borrow_mut()
            .define("clock".to_string(), Value::Callable(Rc::new(Clock)));

        interpreter
    }

    fn is_truthy(v: &Value) -> bool {
        match v {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }

    // Report an evaluation error
    fn error<T>(token: &Token, message: &str) -> InterpreterResult<T> {
        if token.token_type == TokenType::Eof {
            Err(ControlFlow::RuntimeError(RuntimeError::new(
                token.line,
                format!("Error at end: {}", message),
            )))
        } else {
            Err(ControlFlow::RuntimeError(RuntimeError::new(
                token.line,
                format!("Error at '{}': {}", token.lexeme, message),
            )))
        }
    }

    fn as_number(operator: &Token, v: &Value) -> InterpreterResult<f64> {
        match v {
            Value::Float(n) => Ok(*n),
            Value::Integer(i) => Ok(*i as f64),
            _ => Self::error(operator, &format!("Operand must be a number for {}", operator.lexeme)),
        }
    }

    pub fn resolve(&mut self, expression: &mut Expr, depth: usize) {
        if let Expr::Variable { depth: expr_depth, .. } = expression {
            *expr_depth = Depth::Resolved(depth);
        } else if let Expr::Assign { depth: expr_depth, .. } = expression {
            *expr_depth = Depth::Resolved(depth);
        }
    }

    pub fn execute_block(&mut self, statements: &[Statement], environment: EnvRef) -> InterpreterResult<Value> {
        // Create a new environment enclosed by the current one
        let previous_environment = self.environment.clone();
        self.environment = environment;

        // Execute each statement in the block
        for statement in statements {
            self.visit_statement(statement)?;
        }

        // Restore the previous environment
        self.environment = previous_environment;

        Ok(Value::Nil)
    }

    // Interpret (run) a series of statements (can be used for the whole program or a block)
    pub fn interpret(&mut self, statements: &[Statement]) {
        for statement in statements {
            if let Err(ControlFlow::RuntimeError(runtime_error)) = self.visit_statement(&statement) {
                eprintln!("{}", runtime_error);
                std::process::exit(70);
            }
        }
    }

    fn lookup_variable(&mut self, name: &Token, depth: Depth) -> InterpreterResult<Value> {
        match depth {
            Depth::Unresolved => self.globals.borrow().get(&name.lexeme, name.line),
            Depth::Resolved(distance) => self.environment.borrow().get_at(distance, &name.lexeme, name.line),
        }
    }

    fn assign_variable(&mut self, name: &Token, value_expr: &Expr, depth: Depth) -> InterpreterResult<Value> {
        // Evaluate the value expression
        let evaluated_value = self.visit_expression(value_expr)?;

        // Assign the value to the variable at the correct depth
        match depth {
            Depth::Unresolved => {
                self.globals
                    .borrow_mut()
                    .assign(&name.lexeme, evaluated_value.clone(), name.line)?;
            }
            Depth::Resolved(distance) => {
                self.environment
                    .borrow_mut()
                    .assign_at(distance, &name.lexeme, evaluated_value.clone(), name.line)?; // Ensure variable exists
            }
        }

        // Return the assigned value
        Ok(evaluated_value)
    }
}
impl Visitor<InterpreterResult<Value>> for Interpreter {
    fn visit_expression_statement(&mut self, expression: &Expr) -> InterpreterResult<Value> {
        self.visit_expression(expression)
    }

    fn visit_print_statement(&mut self, expression: &Expr) -> InterpreterResult<Value> {
        let value = self.visit_expression(expression)?;
        println!("{}", value);
        Ok(Value::Nil)
    }

    fn visit_block_statement(&mut self, statements: &[Statement]) -> InterpreterResult<Value> {
        // Execute the block in a new environment enclosed by the current one
        self.execute_block(statements, Environment::new(Some(self.environment.clone())))
    }

    fn visit_if_statement(&mut self, condition: &Expr, then_branch: &Statement, else_branch: &Option<Box<Statement>>) -> InterpreterResult<Value> {
        let condition_value = self.visit_expression(condition)?;

        // Execute the then_branch if the condition is truthy, otherwise execute the else_branch if it exists
        if Self::is_truthy(&condition_value) {
            self.visit_statement(then_branch)
        } else if let Some(else_stmt) = else_branch {
            self.visit_statement(else_stmt)
        } else {
            Ok(Value::Nil)
        }
    }

    fn visit_var_statement(&mut self, name: &Token, initializer: &Option<Expr>) -> InterpreterResult<Value> {
        // Evaluate the initializer expression if it exists, otherwise use nil
        let mut value: Value = Value::Nil;
        if let Some(init_expr) = initializer {
            let evaluated_value = self.visit_expression(init_expr)?;
            value = evaluated_value;
        }

        // Define the variable in the current environment
        self.environment
            .borrow_mut()
            .define(name.lexeme.to_string(), value.clone());
        Ok(Value::Nil)
    }

    fn visit_while_statement(&mut self, condition: &Expr, body: &Statement) -> InterpreterResult<Value> {
        // Evaluate the condition and execute the body while the condition is truthy
        while Self::is_truthy(&self.visit_expression(condition)?) {
            self.visit_statement(body)?;
        }

        // Doesn't return anything
        Ok(Value::Nil)
    }

    // Declare and define a function
    fn visit_function_statement(&mut self, statement: &Statement) -> InterpreterResult<Value> {
        // Create a Function from the statement
        let function: Function = Function::from_statement(statement, self.environment.clone())?;

        // Define the function in the current environment
        self.environment
            .borrow_mut()
            .define(function.name().to_string(), Value::Callable(Rc::new(function)));

        Ok(Value::Nil)
    }

    fn visit_class_statement(&mut self, name: &Token, methods: &[Statement]) -> InterpreterResult<Value> {
        // Define the class in the current environment
        self.environment
            .borrow_mut()
            .define(name.lexeme.to_string(), Value::Callable(Rc::new(Class { name: name.lexeme.clone(), methods: methods.to_vec() })));

        Ok(Value::Nil)
    }

    fn visit_return_statement(&mut self, keyword: &Token, value: &Option<Expr>) -> InterpreterResult<Value> {
        // Evaluate the return value expression if it exists, otherwise use nil
        let return_value = if let Some(value_expr) = value {
            self.visit_expression(value_expr)?
        } else {
            Value::Nil
        };

        // Use ControlFlow to signal a return
        Err(ControlFlow::Return(return_value))
    }

    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> InterpreterResult<Value> {
        let left_value = self.visit_expression(left)?;
        let right_value = self.visit_expression(right)?;
        let non_numeric = !matches!(left_value, Value::Float(_) | Value::Integer(_))
            || !matches!(right_value, Value::Float(_) | Value::Integer(_));
        let either_floating =
            matches!(left_value, Value::Float(_)) || matches!(right_value, Value::Float(_));

        match operator.token_type {
            TokenType::Plus => {
                // Handle string concatenation
                if non_numeric {
                    let (Value::Str(str_left), Value::Str(str_right)) = (left_value, right_value) else {
                        return Self::error(operator, "Operands must be two numbers or two strings for '+'");
                    };
                    return Ok(Value::Str(format!("{}{}", str_left, str_right)));
                }
                // Handle numeric addition
                else if either_floating {
                    return Ok(Value::Float(
                        Self::as_number(operator, &left_value)?
                            + Self::as_number(operator, &right_value)?,
                    ));
                } else {
                    let (Value::Integer(num_left), Value::Integer(num_right)) = (left_value, right_value) else {
                        return Self::error(operator, "Operands must be two numbers or two strings for '+'");
                    };
                    return Ok(Value::Integer(num_left + num_right));
                }
            }
            TokenType::Minus => {
                if non_numeric {
                    return Self::error(operator, "Operands must be two numbers for '-'");
                } else if either_floating {
                    return Ok(Value::Float(
                        Self::as_number(operator, &left_value)?
                            - Self::as_number(operator, &right_value)?,
                    ));
                } else {
                    let (Value::Integer(num_left), Value::Integer(num_right)) = (left_value, right_value) else {
                        return Self::error(operator, "Operands must be two integers for '-'");
                    };
                    return Ok(Value::Integer(num_left - num_right));
                }
            }
            TokenType::Star => {
                if non_numeric {
                    return Self::error(operator, "Operands must be two numbers for '*'");
                } else if either_floating {
                    return Ok(Value::Float(
                        Self::as_number(operator, &left_value)?
                            * Self::as_number(operator, &right_value)?,
                    ));
                } else {
                    let (Value::Integer(num_left), Value::Integer(num_right)) = (left_value, right_value) else {
                        return Self::error(operator, "Operands must be two integers for '*'");
                    };
                    return Ok(Value::Integer(num_left * num_right));
                }
            }
            TokenType::Slash => {
                if non_numeric {
                    return Self::error(operator, "Operands must be two numbers for '/'");
                }
                Ok(Value::Float(
                    Self::as_number(operator, &left_value)? / Self::as_number(operator, &right_value)?,
                ))
            }
            TokenType::Greater => {
                let (num_left, num_right) = (
                    Self::as_number(operator, &left_value)?,
                    Self::as_number(operator, &right_value)?,
                );
                Ok(Value::Bool(num_left > num_right))
            }
            TokenType::GreaterEqual => {
                let (num_left, num_right) = (
                    Self::as_number(operator, &left_value)?,
                    Self::as_number(operator, &right_value)?,
                );
                Ok(Value::Bool(num_left >= num_right))
            }
            TokenType::Less => {
                let (num_left, num_right) = (
                    Self::as_number(operator, &left_value)?,
                    Self::as_number(operator, &right_value)?,
                );
                Ok(Value::Bool(num_left < num_right))
            }
            TokenType::LessEqual => {
                let (num_left, num_right) = (
                    Self::as_number(operator, &left_value)?,
                    Self::as_number(operator, &right_value)?,
                );
                Ok(Value::Bool(num_left <= num_right))
            }
            TokenType::EqualEqual => Ok(Value::Bool(is_equal(&left_value, &right_value))),
            TokenType::BangEqual => Ok(Value::Bool(!is_equal(&left_value, &right_value))),
            _ => Self::error(
                operator,
                &format!("Unsupported binary operator: {:?}", operator.token_type),
            ),
        }
    }

    fn visit_literal(&mut self, value: &Token) -> InterpreterResult<Value> {
        // Convert the token's literal to a Value
        let v = match value.literal.as_ref() {
            Some(Literal::Number(n)) => {
                // Distinguish integer vs float based on presence of decimal point in lexeme
                if value.lexeme.contains('.') {
                    Value::Float(*n)
                } else {
                    Value::Integer(*n as isize)
                }
            }
            Some(Literal::String(s)) => Value::Str(s.clone()),
            Some(Literal::Boolean(b)) => Value::Bool(*b),
            Some(Literal::Nil) => Value::Nil,
            None => Value::Nil,
        };
        Ok(v)
    }

    // Evaluate the inner expression
    fn visit_grouping(&mut self, expression: &Expr) -> InterpreterResult<Value> {
        self.visit_expression(expression)
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> InterpreterResult<Value> {
        // Evaluate the right-hand side expression
        let right_value = self.visit_expression(right)?;

        // Apply the unary operator
        match operator.token_type {
            TokenType::Minus => {
                // Return the negated number or error if it's not a number
                if let Value::Float(num) = right_value {
                    return Ok(Value::Float(-num));
                } else if let Value::Integer(num) = right_value {
                    return Ok(Value::Integer(-num));
                } else {
                    return Self::error(operator, "Operand must be a number for unary '-'");
                }
            }
            // Return the logical NOT of the truthiness of the right-hand side
            TokenType::Bang => Ok(Value::Bool(!Self::is_truthy(&right_value))),
            _ => Self::error(
                operator,
                &format!("Unsupported unary operator: {:?}", operator.token_type),
            ),
        }
    }

    fn visit_variable(&mut self, name: &Token, depth: &Depth) -> InterpreterResult<Value> {
        self.lookup_variable(name, *depth)
    }

    fn visit_assign(&mut self, name: &Token, value: &Expr, depth: &Depth) -> InterpreterResult<Value> {
        self.assign_variable(name, value, *depth)
    }

    fn visit_logical_or(&mut self, left: &Expr, right: &Expr) -> InterpreterResult<Value> {
        // Evaluate the left expression
        let left_value = self.visit_expression(left)?;

        // If the left value is truthy, return it, because now we know at least one operand is truthy
        if Self::is_truthy(&left_value) {
            Ok(left_value)
        }
        // Now evaluate and return the right expression
        else {
            self.visit_expression(right)
        }
    }

    fn visit_logical_and(&mut self, left: &Expr, right: &Expr) -> InterpreterResult<Value> {
        // Evaluate the left expression
        let left_value = self.visit_expression(left)?;

        // If the left value is falsy, return it, because now we know at least one operand is falsy
        if !Self::is_truthy(&left_value) {
            Ok(left_value)
        }
        // Now evaluate and return the right expression
        else {
            self.visit_expression(right)
        }
    }

    fn visit_call(&mut self, callee: &Expr, paren: &Token, arguments: &Vec<Expr>) -> InterpreterResult<Value> {
        // Evaluate the callee expression to get the function to call (usually an identifier)
        let Value::Callable(function) = self.visit_expression(callee)? else {
            // Not a callable
            return Self::error(paren, "Can only call functions and classes.");
        };

        // Evaluate each argument expression
        let mut arg_values = Vec::new();
        for arg_expr in arguments {
            let arg_value = self.visit_expression(arg_expr)?;
            arg_values.push(arg_value);
        }

        // Check arity
        if arg_values.len() != function.arity() {
            return Self::error(
                paren,
                &format!(
                    "Expected {} arguments but got {}.",
                    function.arity(),
                    arg_values.len()
                ),
            );
        }

        // Call the function
        Ok(function.call(self, arg_values)?)
    }

    fn visit_lambda(&mut self, params: &Vec<Token>, body: &Vec<Statement>) -> InterpreterResult<Value> {
        // Create a Function representing the lambda
        let lambda_function = Function::new(
            "<lambda>".to_string(),
            params.iter().map(|param| param.lexeme.clone()).collect(),
            // This clones the body statements, which is inefficient but acceptable for this context
            body.clone(),
            self.environment.clone(),
        );

        // Return the lambda as a callable Value
        Ok(Value::Callable(Rc::new(lambda_function)))
    }

    fn visit_get(&mut self, object: &Expr, name: &Token) -> InterpreterResult<Value> {
        let object_value = self.visit_expression(object)?;

        if let Value::Instance(instance) = object_value {
            Ok(instance.get(name)?)
        } else {
            Self::error(name, "Only instances have properties.")
        }
    }
}

fn is_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Integer(x), Value::Integer(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        // No cross-type equality in Lox
        _ => false,
    }
}
