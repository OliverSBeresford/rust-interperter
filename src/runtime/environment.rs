use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::runtime::{ControlFlow, RuntimeError, Value};

// Type for a reference to an Environment wrapped in Rc and RefCell for shared ownership and mutability
pub type EnvRef = Rc<RefCell<Environment>>;

pub type EnvResult<T> = Result<T, ControlFlow>;

#[derive(Debug)]
pub struct Environment {
    // Stores enclosing environment (if any)
    pub enclosing: Option<EnvRef>,

    // Stores variable names and their associated values
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new(enclosing: Option<EnvRef>) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            enclosing,
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str, line: usize) -> EnvResult<Value> {
        // If the variable is found in the current environment, return a cloned value
        if let Some(value) = self.values.get(name) {
            return Ok(value.clone());
        }

        // Otherwise, check the enclosing environment (if any)
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name, line);
        }

        // If the variable is not found, return an error
        Err(ControlFlow::RuntimeError(RuntimeError::new(
            line,
            format!("Undefined variable '{}'.", name),
        )))
    }

    /// Get a variable's value at a specific distance in the environment chain (recursive)
    pub fn get_at(&self, distance: usize, name: &str, line: usize) -> EnvResult<Value> {
        if distance == 0 {
            return self.get(name, line);
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get_at(distance - 1, name, line);
        }

        Err(ControlFlow::RuntimeError(RuntimeError::new(
            line,
            format!("Undefined variable '{}'.", name),
        )))
    }

    pub fn assign(&mut self, name: &str, value: Value, line: usize) -> EnvResult<()> {
        // If the variable exists in the current environment, update its value
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        }

        // Otherwise, check the enclosing environment (if any)
        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.borrow_mut().assign(name, value, line);
        }

        // Variable is not defined in any environment, return an error
        Err(ControlFlow::RuntimeError(RuntimeError::new(
            line,
            format!("Undefined variable '{}'.", name),
        )))
    }

    /// Assign a variable's value at a specific distance in the environment chain (recursive)
    pub fn assign_at(&mut self, distance: usize, name: &str, value: Value, line: usize) -> EnvResult<()> {
        if distance == 0 {
            self.assign(name, value, line)?;
            
            return Ok(())
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign_at(distance - 1, name, value, line);
        }

        Err(ControlFlow::RuntimeError(RuntimeError::new(
            line,
            format!("Undefined variable '{}'.", name),
        )))
    }
}
