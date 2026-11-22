use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::VecDeque;

#[derive(Debug, Deserialize, Serialize)]
pub enum Status {
    Assigned,
    Queued,
    Failed, // will be requeued or assigned to a different worker ig
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

#[derive(Debug, Serialize)]
pub struct TaskQueue {
    content: VecDeque<Task>,
    outputs: Vec<Output>,
}

// Some kinda event broadcast is required here
impl TaskQueue {
    pub fn create_queue() -> TaskQueue {
        TaskQueue {
            content: VecDeque::new(),
            outputs: Vec::new(),
        }
    }

    /// Insert a new task into the queue.
    pub fn new_task(&mut self, mut tsk: Task) {
        tsk.status = Status::Queued;
        self.content.push_back(tsk);
    }

    /// Push a task output (e.g. after execution completes).
    pub fn push_output(&mut self, out: Output) {
        self.outputs.push(out);
    }

    /// Number of tasks currently waiting in the queue.
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Whether the queue currently holds no tasks.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Remove and return the next task, or `None` if the queue is empty.
    pub fn pop_task(&mut self) -> Option<Task> {
        if let Some(task) = self.content.front_mut() {
            task.status = Status::Assigned;
        }
        self.content.pop_front()
    }

    /// Return all stored outputs without clearing them.
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    /// Consume and return all stored outputs, clearing the internal list.
    pub fn take_outputs(&mut self) -> Vec<Output> {
        std::mem::take(&mut self.outputs)
    }
}

