use std::time::{SystemTime, UNIX_EPOCH};
use std::rc::Rc;

use crate::runtime::callable::Callable;
use crate::runtime::control_flow::ControlFlow;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;

/// A native function that returns the current time in seconds since the Unix epoch.
#[derive(Debug)]
pub struct Clock;

impl Callable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(self: Rc<Self>, _interpreter: &mut Interpreter, _args: Vec<Value>) -> Result<Value, ControlFlow> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Ok(Value::Float(now.as_secs_f64()))
    }

    fn to_string(&self) -> String {
        "<native fn clock>".to_string()
    }

    fn name(&self) -> &str {
        "clock"
    }
}
