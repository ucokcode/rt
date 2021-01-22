#![allow(dead_code)]
mod task;
use once_cell::sync::Lazy;
use std::thread;

use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use task::Task;

pub(crate) static GLOBAL: Lazy<(Sender<Task>, Receiver<Task>)> = Lazy::new(|| unbounded());
pub(crate) static WAIT: Lazy<(Sender<()>, Receiver<()>)> = Lazy::new(|| unbounded());

pub mod prelude {
    pub use super::Plugin;
}

pub fn run() {
    let num = num_cpus::get();
    for _ in 0..num {
        thread::spawn(move || {
            while let Ok(mut task) = GLOBAL.1.recv() {
                task.exec();
            }
        });
    }
}

fn _context<'task, T, F>(task: F) -> Receiver<T>
where
    F: FnOnce() -> T + Send + 'task,
    T: Send + 'task,
{
    let (s, r) = bounded(1);

    let f = move || {
        s.send(task()).unwrap();
    };

    let x: Box<dyn FnOnce() + Send> = Box::new(f);
    let x: Box<dyn FnOnce() + Send + 'static> = unsafe { std::mem::transmute(x) };

    let f = Task::new(x);

    GLOBAL.0.send(f).unwrap();
    r
}

fn _no_context<'task, T, F>(task: F)
where
    F: FnOnce() -> T + Send + 'task,
    T: Send + 'task,
{
    let x: Box<dyn FnOnce() + Send> = Box::new(move || {
        task();
    });
    let x: Box<dyn FnOnce() + Send + 'static> = unsafe { std::mem::transmute(x) };
    let f = Task::new(x);
    GLOBAL.0.send(f).unwrap();
}

pub fn wait() {
    (|| WAIT.0.send(()).unwrap()).f().wait();
    WAIT.1.recv().unwrap();
}

pub trait Plugin {
    type Output;
    fn f(self) -> Self::Output;
    fn io(self);
}

pub struct Routine<T>(Receiver<T>);

impl<'task, T> Routine<T>
where
    T: Send + 'task,
{
    pub fn wait(&self) -> T {
        self.0.recv().unwrap()
    }
}

impl<'task, F, T> Plugin for F
where
    F: FnOnce() -> T + Send + 'task,
    T: Send + 'task,
{
    type Output = Routine<T>;
    fn f(self) -> Self::Output {
        Routine(_context(self))
    }

    fn io(self) {
        _no_context(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn once() {
        run();
        let x = (|| 2).f();
        assert_eq!(x.wait(), 2);
    }

    #[test]
    fn wait_of_wait() {
        run();
        let x = (|| 2).f();
        let y = (move || x.wait() * 2).f();
        assert_eq!(y.wait(), 4);
    }

    #[test]
    fn vec_task() {
        run();
        let x = (|| {
            let mut vec = Vec::new();
            for _ in 0..1_000_000 {
                let c = || 1 * 2;
                vec.push(c.f());
            }
            vec.iter().map(|x| x.wait()).sum::<i32>()
        })
        .f();
        assert_eq!(x.wait(), 2_000_000);
    }

    #[test]
    fn io() {
        run();
        let mut x = 2;
        (|| x += 1).io();
        (|| x += 1).io();
        (|| x += 1).io();
        wait();
        assert_eq!(x, 5);
    }
}
