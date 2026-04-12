use std::rc::Rc;
use crate::runtime::class::Class;

#[derive(Debug)]
pub struct Instance {
    pub class: Rc<Class>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Instance { class }
    }
}
