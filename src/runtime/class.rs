use crate::ast::statement::Statement;
use crate::runtime::callable::Callable;
use crate::runtime::control_flow::ControlFlow;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;
use std::rc::Rc;
use crate::runtime::instance::Instance;

pub type FunctionResult<T> = Result<T, ControlFlow>;

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Statement>,
}

impl Class {
    pub fn new(name: String, methods: Vec<Statement>) -> Self {
        Class { name, methods }
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    fn call(self: Rc<Self>, _interpreter: &mut Interpreter, _args: Vec<Value>) -> FunctionResult<Value> {
        return Ok(Value::Instance(Rc::new(Instance::new(self))));
    }

    fn to_string(&self) -> String {
        format!("<class {}>", self.name)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
