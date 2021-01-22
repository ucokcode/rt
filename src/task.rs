use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Ts {
    Pending,
    Active,
    Finish,
}

pub struct Task {
    s: Ts,
    f: Option<Box<dyn FnOnce() + Send + 'static>>,
}

use Ts::*;

impl Task {
    pub fn new(f: Box<dyn FnOnce() + Send + 'static>) -> Task {
        let task = Task {
            s: Pending,
            f: Some(f),
        };
        task
    }

    pub fn is_active(&self) -> bool {
        self.s == Active
    }

    pub fn exec(&mut self) {
        if let Some(_f) = self.f.take() {
            self.s = Active;
            _f();
            self.s = Finish;
        }
    }
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Task").field(&"ƒ").finish()
    }
}
