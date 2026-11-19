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
        let mut workers = self.workers.lock().unwrap();
        let mut tasks = self.tasks.lock().unwrap();

        for worker in &mut workers.workers {
            if !worker.available {
                continue;
            }
            let Some(task) = tasks.content.pop_front() else {
                break;
            };
            worker.available = false;
            let worker_id = worker.id;
            let tasks = Arc::clone(&self.tasks);
            let workers = Arc::clone(&self.workers);

            tokio::spawn(async move {
                let result = async {
                    let id = i32::try_from(task.id).map_err(|err| err.to_string())?;
                    let mut client = task_rpc::task_client::TaskClient::connect(format!(
                        "http://localhost:{worker_id}"
                    ))
                    .await
                    .map_err(|err| err.to_string())?;
                    client
                        .fetch_task(task_rpc::TaskContent {
                            id,
                            command: task.command,
                            args: task.args,
                        })
                        .await
                        .map(|response| response.into_inner())
                        .map_err(|err| err.to_string())
                }
                .await;

                let output = match result {
                    Ok(result) => result.content.join("\n"),
                    Err(error) => {
                        // An RPC failure does not prove execution stopped. Keep the worker reserved.
                        tasks.lock().unwrap().outputs.push(Output {
                            output: format!("Dispatch failed for worker {worker_id}: {error}"),
                            task_id: task.id,
                        });
                        return;
                    }
                };
                tasks.lock().unwrap().outputs.push(Output {
                    output,
                    task_id: task.id,
                });
                // The worker responds only after execution; the scheduler never waits here.
                let mut workers = workers.lock().unwrap();
                if let Some(worker) = workers.workers.iter_mut().find(|w| w.id == worker_id) {
                    worker.available = true;
                }
            });
        }
    }
}
