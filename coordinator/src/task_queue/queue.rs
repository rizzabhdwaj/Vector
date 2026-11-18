use serde::Deserialize;
use std::collections::VecDeque;
#[derive(Debug, Deserialize)]
enum Status {
    compleated,
    assigned,
    failed,
}

#[derive(Debug, Deserialize)]
struct task {
    id: u32,
    command: String,
    args: Vec<String>,
    status: Status,
}
#[derive(Debug, Default, Deserialize)]
pub struct TaskQueue {
    content: VecDeque<task>,
}

impl TaskQueue {
    pub fn create_queue() -> TaskQueue {
        TaskQueue::default()
    }
    pub fn new_task(&mut self, task_data: &str) -> Result<(), serde_json::Error> {
        let tsk: task = serde_json::from_str(task_data)?;
        self.content.push_back(tsk);
        Ok(())
    }
}
