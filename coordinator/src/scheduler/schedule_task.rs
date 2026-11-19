//access the TaskQueue.top() task and scedule it to one of the workers
//
//
use crate::task_queue::queue::TaskQueue;
use crate::worker::workers::Workforce;
use std::sync::{Arc, Mutex};

pub struct Scheduler {
    tasks: Arc<Mutex<TaskQueue>>,
    workers: Arc<Mutex<Workforce>>,
}

impl Scheduler {
    pub fn init_scheduler(tasks: Arc<Mutex<TaskQueue>>, workers: Arc<Mutex<Workforce>>) -> Self {
        Self { tasks, workers }
    }
}
