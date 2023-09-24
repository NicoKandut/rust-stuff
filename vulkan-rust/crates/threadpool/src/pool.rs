use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};

use crate::worker::{Command, Worker};

pub struct ThreadPool {
    sender: Sender<Command>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(thread_name: &str, size: usize) -> Self {
        let (sender, receiver) = channel();
        let shared_receiver = Arc::new(Mutex::new(receiver));
        let workers = (0..size)
            .map(|index| {
                Worker::new(
                    format!("{thread_name}_{index}"),
                    Arc::clone(&shared_receiver),
                )
            })
            .collect();

        Self { sender, workers }
    }

    pub fn execute<T>(&self, task: T)
    where
        T: FnOnce() + Send + 'static,
    {
        self.sender
            .send(Command::Execute(Box::new(task)))
            .expect("Sending work failed");
    }

    // pub fn execute_many<T>(&self, tasks: Vec<T>)
    // where
    //     T: FnOnce() + Send + 'static,
    // {
    //     for task in tasks {
    //         self.execute(task);
    //     }
    // }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _worker in &self.workers {
            self.sender
                .send(Command::Exit)
                .expect("Sending exit command failed");
        }
        self.workers.drain(..).for_each(|worker| worker.join());
    }
}
