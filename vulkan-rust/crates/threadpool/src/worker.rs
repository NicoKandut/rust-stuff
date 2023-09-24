use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::{self, JoinHandle},
};

pub(crate) type Task = Box<dyn FnOnce() + Send + 'static>;

pub(crate) enum Command {
    Execute(Task),
    Exit,
}
pub(crate) struct Worker {
    handle: JoinHandle<()>,
}

impl Worker {
    pub(crate) fn new(name: String, receiver: Arc<Mutex<Receiver<Command>>>) -> Self {
        let handle = thread::Builder::new()
            .name(name)
            .spawn(move || loop {
                match receiver.lock().unwrap().recv() {
                    Ok(command) => match command {
                        Command::Execute(task) => task(),
                        Command::Exit => break,
                    },
                    Err(_) => break,
                }
            })
            .expect("Failed to spawn thread");

        Self { handle }
    }

    pub(crate) fn join(self) {
        self.handle.join().expect("Failed to join thread");
    }
}
