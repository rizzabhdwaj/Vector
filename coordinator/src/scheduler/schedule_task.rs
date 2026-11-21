use crate::task_queue::queue::{Output, Status, TaskQueue};
use crate::worker::workers::{WorkerStatus, Workforce};
use sqlx::PgPool;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;
use tokio::sync::mpsc;
use crate::scheduler::event::Event;

mod task_rpc {
    tonic::include_proto!("task");
}

// A poisoned lock means some thread panicked while holding it. The scheduler loop must
// survive that, so recover the guard instead of unwrapping (which would panic the loop).
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

pub struct Scheduler {
    tasks: Arc<Mutex<TaskQueue>>,
    workers: Arc<Mutex<Workforce>>,
    event_bus_sender: Option<tokio::sync::mpsc::Sender<Event>>,
}

impl Scheduler {
    pub fn init_scheduler(
        tasks: Arc<Mutex<TaskQueue>>,
        workers: Arc<Mutex<Workforce>>,
        event_bus_sender: Option<tokio::sync::mpsc::Sender<Event>>,
    ) -> Self {
        Self {
            tasks,
            workers,
            event_bus_sender,
        }
    }

    pub async fn run(self) {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            interval.tick().await;
            self.schedule_available().await;
        }
    }

    async fn schedule_available(&self) {
        let mut workers = lock(&self.workers);
        let mut tasks = lock(&self.tasks);

        for worker in &mut workers.workers {
            if worker.curr_status != WorkerStatus::Free {
                continue;
            }
            let Some(task) = tasks.pop_task() else {
                break;
            };
            worker.curr_status = WorkerStatus::Busy;
            let worker_id = worker.id;
            let tasks = Arc::clone(&self.tasks);
            let workers = Arc::clone(&self.workers);

            // Clone sender for the spawned task
            let sender = self.event_bus_sender.clone();

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
                        if let Some(s) = &sender {
                            let _ = s.send(Event::TaskAssigned {
                                id: task.id,
                                worker_id: worker_id,
                            }).await;
                        }
                        lock(&tasks).push_output(Output {
                            output: format!("Dispatch failed for worker {worker_id}: {error}"),
                            task_id: task.id,
                        });
                        return;
                    }
                };
                if let Some(s) = &sender {
                    let _ = s.send(Event::TaskFinished {
                        id: task.id,
                        worker_id,
                        output: output.clone(),
                        success: true,
                    }).await;
                }
                lock(&tasks).push_output(Output {
                    output,
                    task_id: task.id,
                });
                // The worker responds only after execution; the scheduler never waits here.
                let mut workers = lock(&workers);
                if let Some(w) = workers.workers.iter_mut().find(|w| w.id == worker_id) {
                    w.curr_status = WorkerStatus::Free;
                }
            });
        }
    }
}