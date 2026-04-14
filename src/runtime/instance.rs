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
        // First, check if the property exists in the instance's fields
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Ok(value.clone());
        }
        // If not found, return a default value (e.g., Nil)
        Err(ControlFlow::RuntimeError(RuntimeError::new(
            name.line,
            format!("Undefined property '{}'.", name.lexeme),
        )))
    }
}
