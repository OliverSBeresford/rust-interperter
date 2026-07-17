use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    runtime::{Callable, ControlFlow, EnvRef, Environment, Instance, Interpreter, RuntimeError, Value},
    ast::Statement,
};

pub type FunctionResult<T> = Result<T, ControlFlow>;

#[derive(Debug, Clone)]
pub struct Function {
    name: String,
    pub params: Vec<String>,
    pub body: Vec<Rc<Statement>>,
    pub closure: EnvRef,
    is_initializer: bool,
}

impl Function {
    // Create a Function from a Statement::Function
    pub fn from_statement(stmt: Rc<Statement>, closure: EnvRef, is_initializer: bool) -> FunctionResult<Self> {
        if let Statement::Function { name, params, body } = &*stmt {
            Ok(Function {
                name: name.lexeme.clone(),
                params: params.iter().map(|param| param.lexeme.clone()).collect(),
                // This clones the body statements, which is inefficient but acceptable for this context (see other branch for version without clone)
                body: body.clone(),
                closure,
                is_initializer,
            })
        } else {
            // This should not happen if used correctly (even if the user makes a mistake)
            Err(ControlFlow::RuntimeError(RuntimeError::new(
                0,
                "Expected a function statement.".to_string(),
            )))
        }
    }

    pub fn new(name: String, params: Vec<String>, body: Vec<Rc<Statement>>, closure: EnvRef, is_initializer: bool) -> Self {
        Function { name, params, body, closure, is_initializer }
    }

    pub fn bind(&self, instance: Rc<RefCell<Instance>>) -> Self {
        let bound_closure: EnvRef = Environment::new(Some(self.closure.clone()));
        bound_closure
            .borrow_mut()
            .define("this".to_string(), Value::Instance(instance));

        Function {
            name: self.name.clone(),
            params: self.params.clone(),
            body: self.body.clone(),
            closure: bound_closure,
            is_initializer: self.is_initializer,
        }
    }

    fn get_instance_from_closure(&self) -> FunctionResult<Value> {
        Ok(self.closure
            .borrow()
            .get("this", 0)
            .expect("Expected 'this' to be defined in the closure.")
            .clone())
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
        match interpreter.execute_block(self.body.clone(), environment) {
            Ok(_) => {}
            Err(ControlFlow::Return(return_value)) => {
                interpreter.environment = previous_environment;
                
                // If the function is an initializer, return the instance bound to "this" in the closure; otherwise, return the return value
                if self.is_initializer {
                    return self.get_instance_from_closure();
                }

                return Ok(return_value);
            }
            Err(ControlFlow::RuntimeError(runtime_error)) => {
                return Err(ControlFlow::RuntimeError(runtime_error));
            }
        }

        // If the function is an initializer, return the instance bound to "this" in the closure; otherwise, return Nil
        if self.is_initializer {
            return self.get_instance_from_closure();
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
