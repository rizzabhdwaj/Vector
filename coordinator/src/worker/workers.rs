use crate::api::grpc_server::heartbeat::HeartbeatRequest;
use crate::task_queue::queue::Task;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WorkerStatus {
    Busy,
    Free,
    Dead,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct Worker {
    pub(crate) id: i32,
    pub(crate) curr_status: WorkerStatus,
    /// Tasks assigned to this worker that have not finished yet.
    pub(crate) task_assigned: Vec<Task>,
    #[serde(skip)]
    pub(crate) last_seen: Instant,
}

// we need a way to keep limited workers and update them as per heartbeat
#[derive(Default, serde::Serialize)]
pub struct Workforce {
    pub(crate) workers: Vec<Worker>,
}

impl Workforce {
    pub fn update_worker(&mut self, heartbeat: HeartbeatRequest) {
        for i in &mut self.workers {
            if i.id == heartbeat.worker_id {
                // The scheduler owns status after registration.
                let gap = i.last_seen.elapsed();
                i.last_seen = Instant::now();

                if gap > Duration::from_secs(6) {
                    println!("worker {} heartbeat gap: {:?}", i.id, gap);
                }

                // If the worker was previously marked dead, a heartbeat revives it.
                if i.curr_status == WorkerStatus::Dead {
                    i.curr_status = WorkerStatus::Free;
                    println!("worker {} revived from dead state", i.id);
                }

                return;
            }
        }
        self.workers.push(Worker {
            id: heartbeat.worker_id,
            curr_status: if heartbeat.available {
                WorkerStatus::Free
            } else {
                WorkerStatus::Busy
            },
            task_assigned: Vec::new(),
            last_seen: Instant::now(),
        });
    }

    /// Drop a task from a worker once it is no longer pending (completed or failed).
    pub(crate) fn finish_task(&mut self, worker_id: i32, task_id: u32) {
        if let Some(worker) = self.workers.iter_mut().find(|w| w.id == worker_id) {
            worker.task_assigned.retain(|task| task.id != task_id);
        }
    }

    pub fn init_workforce() -> Workforce {
        Workforce::default()
    }

    /*
        A Reaper as Claude describes it "It’s a background task in the coordinator that wakes up every second, checks how long it’s been since each worker’s last heartbeat, and marks the worker dead once that exceeds a timeout"  Scary

    */

    pub fn reap_dead(&mut self, timeout: Duration) -> Vec<i32> {
        let now = Instant::now();
        let mut dead = Vec::new();
        for w in &mut self.workers {
            if w.curr_status != WorkerStatus::Dead && now.duration_since(w.last_seen) > timeout {
                w.curr_status = WorkerStatus::Dead;
                dead.push(w.id);
            }
        }
        dead
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finishing_a_task_removes_it_from_the_worker() {
        let mut workforce = Workforce::init_workforce();
        workforce.update_worker(HeartbeatRequest {
            worker_id: 5001,
            available: true,
        });
        for id in [7, 8] {
            let task: Task = serde_json::from_str(&format!(
                r#"{{"id":{id},"command":"echo","args":[],"status":"Queued"}}"#
            ))
            .unwrap();
            workforce.workers[0].task_assigned.push(task);
        }

        workforce.finish_task(5001, 7);

        assert_eq!(
            workforce.workers[0]
                .task_assigned
                .iter()
                .map(|task| task.id)
                .collect::<Vec<_>>(),
            vec![8]
        );
    }
}
