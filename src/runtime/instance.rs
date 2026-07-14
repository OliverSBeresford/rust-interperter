use std::rc::Rc;
use std::collections::HashMap;
use crate::runtime::class::Class;
use crate::runtime::value::Value;
use crate::runtime::runtime_error::RuntimeError;
use crate::lexer::token::Token;
use crate::runtime::control_flow::ControlFlow;

#[derive(Debug)]
pub struct Instance {
    pub class: Rc<Class>,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Instance { class, fields: HashMap::new() }
    }

    pub fn get(&self, name: &Token) -> Result<Value, ControlFlow> {
        // First, check if the field exists in the instance's fields
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Ok(value.clone());
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
