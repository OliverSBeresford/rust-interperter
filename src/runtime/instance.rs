use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;

use crate::{
    runtime::{Class, ControlFlow, Function, RuntimeError, Value, Callable, Interpreter},
    lexer::Token
};

#[derive(Debug)]
pub struct Instance {
    pub class: Rc<Class>,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Instance { class, fields: HashMap::new() }
    }

    pub fn get(&self, instance: Rc<RefCell<Instance>>, name: &Token, interpreter: &mut Interpreter) -> Result<Value, ControlFlow> {
        // First, check if the field exists in the instance's fields
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Ok(value.clone());
        }

        // If not found in fields, check if it's a method in the class
        if let Some(method) = self.class.find_method(&name.lexeme) {
            // Bind the method to the instance and return it as a callable value
            let bound_method: Function = method.bind(instance.clone());
            let bound_method_rc = Rc::new(bound_method);

            // If the method is a getter, call it immediately and return the result
            if bound_method_rc.is_getter {
                let result = bound_method_rc.clone().call(interpreter, Vec::new())?;
                return Ok(result);
            }

            // Return the method as a callable value
            return Ok(Value::Callable(bound_method_rc));
        }

        // If not found, return an error indicating that the property is undefined
        Err(ControlFlow::RuntimeError(RuntimeError::new(
            name.line,
            format!("Undefined property '{}'.", name.lexeme),
        )))
    }

    pub fn set(&mut self, name: &Token, value: Value) -> Result<(), ControlFlow> {
        // Set the property in the instance's fields
        self.fields.insert(name.lexeme.clone(), value);
        Ok(())
    }
}
