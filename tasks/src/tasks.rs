use std::{
    collections::VecDeque,
    sync::{mpsc::channel, Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct Queue<Input, Output>
where
    Input: Send + Copy,
    Output: Send + Copy,
{
    threads: Vec<JoinHandle<()>>,
    input: Arc<Mutex<VecDeque<Input>>>,
    output: Arc<Mutex<VecDeque<Output>>>,
}

impl<Input, Output> Queue<Input, Output>
where
    Input: Send + Copy,
    Output: Send + Copy,
{
    pub fn new(worker_count: usize, work: fn(Input) -> Output) -> Self {
        let input = Arc::new(Mutex::new(VecDeque::new()));
        let output = Arc::new(Mutex::new(VecDeque::new()));
        let (work_sender, work_receiver) = channel();
        let mut threads = Vec::new();
        for i in 0..worker_count {
            let thread = thread::spawn(|| {
                if let Some(task) = { input.lock().unwrap().pop_front() } {
                    let result = work(task);
                    output.lock().unwrap().push_back(result);
                };
            });
            threads.push(thread)
        }

        Queue {
            threads,
            input,
            output,
        }
    }

    pub fn submit_input(&mut self, new_tasks: Vec<Input>) {
        let mut input_queue = self.input.lock().unwrap();
        input_queue.extend(new_tasks);
    }

    pub fn retrieve_output(&mut self) -> Vec<Output> {
        let mut output_queue = self.output.lock().unwrap();
        output_queue.drain(..).collect()
    }
}
