// mod tasks;

use std::{
    collections::VecDeque,
    marker::Send,
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

#[derive(Clone, Copy)]
struct TaskDescription {}

#[derive(Clone, Copy)]
struct TaskResult {
    task: TaskDescription,
    done_by: usize,
    res: f32,
}

fn create_work_queue<I: 'static + Clone + Send, O: 'static + Clone + Send>(
    work_function: fn(&I) -> O,
) -> (Vec<JoinHandle<()>>, Arc<Mutex<VecDeque<I>>>, Receiver<O>) {
    let work_queue = Arc::new(Mutex::new(VecDeque::<I>::new()));
    let (result_sender, result_receiver) = channel::<O>();

    let mut threads = Vec::new();
    for _ in 0..8 {
        let recv = work_queue.clone();
        let send = result_sender.clone();
        let thread = thread::spawn(move || loop {
            if let Some(task) = { recv.lock().unwrap().pop_front() } {
                let result = work_function(&task);
                match send.send(result) {
                    Ok(_) => (),
                    Err(_) => {
                        break;
                    }
                }
            }
            thread::yield_now();
        });
        threads.push(thread)
    }

    (threads, work_queue.clone(), result_receiver)
}

fn create_task_result(desc: &TaskDescription) -> TaskResult {
    let mut res = 0.;
    for i in 0..1000 {
        res += (i as f32).sqrt();
    }

    TaskResult {
        task: *desc,
        done_by: 0,
        res,
    }
}

fn main() {
    let (_threads, input_sender, output_receiver) = create_work_queue(create_task_result);

    /////////

    let task_count = 100000;

    let input = vec![TaskDescription {}; task_count];

    {
        input_sender.lock().unwrap().extend(input);
    }

    let mut output = Vec::<TaskResult>::new();
    while output.len() < task_count {
        output.extend(output_receiver.recv());
    }

    let mut count = [0; 4];

    for result in output {
        count[result.done_by] += 1;
    }

    println!("Done");
    println!("Counts: {:?}", count);
}
