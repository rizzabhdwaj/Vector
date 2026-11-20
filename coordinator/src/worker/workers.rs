use crate::grpc_server::heartbeat::HeartbeatRequest;
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
            last_seen: Instant::now(),
        });
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
