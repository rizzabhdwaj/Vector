use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
#[derive(Debug, Deserialize, Serialize)]
enum Status {
    Compleated,
    Assigned,
    Failed,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub(crate) id: u32,
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
    status: Status,
}
#[derive(Debug, Serialize)]
pub struct Output {
    pub(crate) output: String,
    pub(crate) task_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TaskQueue {
    pub(crate) content: VecDeque<Task>,
    #[serde(skip)]
    pub(crate) outputs: Vec<Output>,
}

impl TaskQueue {
    pub fn create_queue() -> TaskQueue {
        TaskQueue::default()
    }
    pub fn new_task(&mut self, tsk: Task) {
        self.content.push_back(tsk);
    }
}
