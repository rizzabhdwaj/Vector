use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
#[derive(Debug, Deserialize, Serialize)]
enum Status {
    Compleated,
    Assigned,
    Failed,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct task {
    id: u32,
    command: String,
    args: Vec<String>,
    status: Status,
}
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TaskQueue {
    content: VecDeque<task>,
}

impl TaskQueue {
    pub fn create_queue() -> TaskQueue {
        TaskQueue::default()
    }
    pub fn new_task(&mut self, tsk: task) {
        self.content.push_back(tsk);
    }
}
