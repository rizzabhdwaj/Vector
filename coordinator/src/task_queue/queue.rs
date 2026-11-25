use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Task execution time limit, in seconds, when the request omits it.
pub(crate) const DEFAULT_TASK_TIMELIMIT: u32 = 15;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Status {
    Running,
    Queued,
    Failed, // will be requeued or assigned to a different worker ig
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Task {
    pub(crate) id: u32,
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
    #[serde(default = "default_task_timelimit")]
    pub(crate) timelimit: u32,
    status: Status,
}

fn default_task_timelimit() -> u32 {
    DEFAULT_TASK_TIMELIMIT
}
#[derive(Clone, Debug, Serialize)]
pub struct Output {
    pub(crate) output: String,
    pub(crate) task_id: u32,
}

#[derive(Debug, Serialize)]
pub struct TaskQueue {
    content: VecDeque<Task>,
    re_scheduled: VecDeque<Task>,

    outputs: Vec<Output>,
}

// Some kinda event broadcast is required here
impl TaskQueue {
    pub fn create_queue() -> TaskQueue {
        TaskQueue {
            content: VecDeque::new(),
            outputs: Vec::new(),
            re_scheduled: VecDeque::new(),
        }
    }

    /// Insert a new task into the queue.
    pub fn new_task(&mut self, mut tsk: Task) {
        tsk.status = Status::Queued;
        self.content.push_back(tsk);
    }
    pub fn re_schedule(&mut self, tsk: Task) {
        self.re_scheduled.push_back(tsk);
    }
    /// Push a task output (e.g. after execution completes).
    pub fn push_output(&mut self, out: Output) {
        self.outputs.push(out);
    }

    /// All tasks currently in the queue (queued or running).
    pub fn tasks(&self) -> &VecDeque<Task> {
        &self.content
    }

    /// Remove and return the next task, or `None` if the queue is empty.
    pub fn pop_task(&mut self) -> Option<Task> {
        if let Some(task) = self.content.front_mut() {
            task.status = Status::Running;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timelimit_defaults_to_fifteen_seconds() {
        let task: Task =
            serde_json::from_str(r#"{"id":1,"command":"echo","args":[],"status":"Queued"}"#)
                .unwrap();
        assert_eq!(task.timelimit, DEFAULT_TASK_TIMELIMIT);
    }

    #[test]
    fn timelimit_from_request_is_preserved() {
        let task: Task = serde_json::from_str(
            r#"{"id":1,"command":"echo","args":[],"status":"Queued","timelimit":42}"#,
        )
        .unwrap();
        assert_eq!(task.timelimit, 42);
    }
}
