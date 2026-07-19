use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    runtime::{Callable, ControlFlow, Function, Instance, Interpreter, Value, RuntimeError},
    lexer::Token
};

pub type FunctionResult<T> = Result<T, ControlFlow>;

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub superclass: Option<Rc<Class>>,
    pub methods: HashMap<String, Rc<Function>>,
    pub static_fields: HashMap<String, Value>,
    pub static_methods: HashMap<String, Rc<Function>>,
}

impl Class {
    pub fn new(name: String, superclass: Option<Rc<Class>>, methods: HashMap<String, Rc<Function>>, static_fields: HashMap<String, Value>, static_methods: HashMap<String, Rc<Function>>) -> Self {
        Class { name, superclass, methods, static_fields, static_methods }
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<Function>> {
        self.methods.get(name).cloned()
    }

    pub fn get_static_method(&self, name: &str) -> Option<Rc<Function>> {
        self.static_methods.get(name).cloned()
    }

    pub fn get_static_field(&self, name: &str) -> Option<Value> {
        self.static_fields.get(name).cloned()
    }

    pub fn get(self: Rc<Self>, name: &Token) -> Result<Value, ControlFlow> {
        // Check if it's a static field
        if let Some(value) = self.get_static_field(&name.lexeme) {
            return Ok(value);
        }

        // Check if it's a static method
        if let Some(method) = self.get_static_method(&name.lexeme) {
            return Ok(Value::Callable(Rc::new(method.bind_class(self.clone()))));
        }

        Err(ControlFlow::RuntimeError(RuntimeError::new(name.line, format!("Undefined static property '{}'.", name.lexeme))))
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        // The arity of a class is determined by the arity of its initializer (constructor) method, if it exists.
        if let Some(initializer) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
    }

    fn call(self: Rc<Self>, interpreter: &mut Interpreter, args: Vec<Value>) -> FunctionResult<Value> {
        // Create a new instance of the class
        let instance: Instance = Instance::new(self.clone());
        let instance_ref = Rc::new(RefCell::new(instance));

        if let Some(initializer) = self.find_method("init") {
            // Bind the initializer to the instance (for use of "this") and call it with the provided arguments
            let bound_initializer = initializer.bind(instance_ref.clone());

            Rc::new(bound_initializer).call(interpreter, args)?;
        }

        return Ok(Value::Instance(instance_ref.clone()));
    }

    fn to_string(&self) -> String {
        format!("<class {}>", self.name)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn into_any_rc(self: Rc<Self>) -> Rc<dyn std::any::Any>
        where Self: 'static
    {
        self
    }
}
