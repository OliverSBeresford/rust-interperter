use crate::runtime::callable::Callable;
use crate::runtime::control_flow::ControlFlow;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;
use std::collections::HashMap;
use crate::runtime::function::Function;
use std::rc::Rc;
use std::cell::RefCell;
use crate::runtime::instance::Instance;

pub type FunctionResult<T> = Result<T, ControlFlow>;

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<String, Rc<Function>>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Rc<Function>>) -> Self {
        Class { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<Function>> {
        self.methods.get(name).cloned()
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
}
