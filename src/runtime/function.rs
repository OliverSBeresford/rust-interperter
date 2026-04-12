use crate::ast::statement::Statement;
use crate::runtime::callable::Callable;
use crate::runtime::control_flow::ControlFlow;
use crate::runtime::environment::{EnvRef, Environment};
use crate::runtime::interpreter::Interpreter;
use crate::runtime::RuntimeError;
use crate::runtime::value::Value;
use std::rc::Rc;

pub type FunctionResult<T> = Result<T, ControlFlow>;

#[derive(Debug)]
pub struct Function {
    name: String,
    params: Vec<String>,
    body: Vec<Statement>,
    closure: EnvRef,
}

impl Function {
    // Create a Function from a Statement::Function
    pub fn from_statement(stmt: &Statement, closure: EnvRef) -> FunctionResult<Self> {
        if let Statement::Function { name, params, body } = stmt {
            Ok(Function {
                name: name.lexeme.clone(),
                params: params.iter().map(|param| param.lexeme.clone()).collect(),
                // This clones the body statements, which is inefficient but acceptable for this context (see other branch for version without clone)
                body: body.clone(),
                closure,
            })
        } else {
            // This should not happen if used correctly (even if the user makes a mistake)
            Err(ControlFlow::RuntimeError(RuntimeError::new(
                0,
                "Expected a function statement.".to_string(),
            )))
        }
    }

    pub fn new(name: String, params: Vec<String>, body: Vec<Statement>, closure: EnvRef) -> Self {
        Function { name, params, body, closure }
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(self: Rc<Self>, interpreter: &mut Interpreter, args: Vec<Value>) -> FunctionResult<Value> {
        let previous_environment = interpreter.environment.clone();

        let environment: EnvRef = Environment::new(Some(self.closure.clone()));

        // Loop through params and args simultaneously (using zip) and define them in the new environment
        for (param, arg) in self.params.iter().zip(args.into_iter()) {
            environment.borrow_mut().define(param.clone(), arg);
        }

        // Execute the function body in the new environment, handling return values via ControlFlow
        match interpreter.execute_block(&self.body, environment) {
            Ok(_) => {}
            Err(ControlFlow::Return(return_value)) => {
                interpreter.environment = previous_environment;
                return Ok(return_value);
            }
            Err(ControlFlow::RuntimeError(runtime_error)) => {
                return Err(ControlFlow::RuntimeError(runtime_error));
            }
        }

        Ok(Value::Nil)
    }

    fn to_string(&self) -> String {
        format!("<fn {}>", self.name)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
