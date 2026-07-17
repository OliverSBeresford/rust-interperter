use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::any::Any;

use crate::runtime::{Callable, ControlFlow, Interpreter, Value};

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

    fn into_any_rc(self: Rc<Self>) -> Rc<dyn Any>
        where Self: 'static
    {
        self
    }
}
