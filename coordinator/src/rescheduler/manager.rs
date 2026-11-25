use crate::task_queue::queue::TaskQueue;
use crate::worker::workers::{WorkerStatus, Workforce};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;
use tokio::time::interval;

pub(crate) const TIMEOUT: Duration = Duration::from_secs(5);

#[allow(non_camel_case_types)]
pub(crate) struct orphan_mgr {
    workers: Arc<Mutex<Workforce>>,
    tasks: Arc<Mutex<TaskQueue>>,
}

impl orphan_mgr {
    pub(crate) fn new(workers: Arc<Mutex<Workforce>>, tasks: Arc<Mutex<TaskQueue>>) -> Self {
        Self { workers, tasks }
    }

    pub(crate) async fn run(self) {
        let mut ticker = interval(TIMEOUT);
        loop {
            ticker.tick().await;

            let mut dead_tasks = Vec::new();
            {
                let mut workers = self.workers.lock().unwrap_or_else(PoisonError::into_inner);
                for worker in workers.workers.iter_mut() {
                    if worker.curr_status == WorkerStatus::Dead {
                        dead_tasks.extend(worker.task_assigned.drain(..));
                    }
                }
            }

            if !dead_tasks.is_empty() {
                let mut tasks = self.tasks.lock().unwrap_or_else(PoisonError::into_inner);
                for task in dead_tasks {
                    tasks.re_schedule(task);
                }
            }
        }
    }
}
