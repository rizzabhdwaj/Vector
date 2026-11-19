use crate::task_queue::queue::{Output, TaskQueue};
use crate::worker::workers::Workforce;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod task_rpc {
    tonic::include_proto!("task");
}

pub struct Scheduler {
    tasks: Arc<Mutex<TaskQueue>>,
    workers: Arc<Mutex<Workforce>>,
}

impl Scheduler {
    pub fn init_scheduler(tasks: Arc<Mutex<TaskQueue>>, workers: Arc<Mutex<Workforce>>) -> Self {
        Self { tasks, workers }
    }

    pub async fn run(self) {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            interval.tick().await;
            self.schedule_available().await;
        }
    }

    async fn schedule_available(&self) {
        let mut assignments = Vec::new();
        {
            let mut workers = self.workers.lock().unwrap();
            let mut tasks = self.tasks.lock().unwrap();

            for worker in &mut workers.workers {
                if worker.available {
                    let Some(task) = tasks.content.pop_front() else {
                        break;
                    };
                    worker.available = false;
                    assignments.push((worker.id, task));
                }
            }
        }

        // Release shared-state locks before making network requests.
        for (worker_id, task) in assignments {
            let mut client =
                task_rpc::task_client::TaskClient::connect(format!("http://localhost:{worker_id}"))
                    .await
                    .unwrap();

            let result = client
                .fetch_task(task_rpc::TaskContent {
                    id: task.id.try_into().unwrap(),
                    command: task.command,
                    args: task.args,
                })
                .await
                .unwrap();
            let result = result.into_inner();
            self.tasks.lock().unwrap().outputs.push(Output {
                output: result.content.join("\n"),
                task_id: task.id,
            });

            let mut workers = self.workers.lock().unwrap();
            for worker in &mut workers.workers {
                if worker.id == worker_id {
                    worker.available = true;
                    break;
                }
            }
        }
    }
}
