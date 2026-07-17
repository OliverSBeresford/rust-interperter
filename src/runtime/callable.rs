use std::fmt::Debug;
use std::rc::Rc;

use crate::runtime::{ControlFlow, Interpreter, Value};

pub trait Callable: Debug {
    fn arity(&self) -> usize;
    fn call(self: Rc<Self>, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, ControlFlow>;
    fn to_string(&self) -> String;
    fn name(&self) -> &str;
}
