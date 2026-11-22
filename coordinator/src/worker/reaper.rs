use crate::worker::workers::Workforce;
use std::sync::{Arc, Mutex};
use std::time::Duration;
/*
    A Reaper as Claude describes it "It’s a background task in the coordinator that wakes up every second, checks how long it’s been since each worker’s last heartbeat, and marks the worker dead once that exceeds a timeout"  Scary

*/

pub struct Reaper {
    workers: Arc<Mutex<Workforce>>,
    timeout: Duration,
}

impl Reaper {
    pub fn new(workers: Arc<Mutex<Workforce>>, timeout: Duration) -> Reaper {
        Self {
            workers,
            timeout,
        }
    }

    pub async fn run(self) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let dead = self.workers.lock().unwrap().reap_dead(self.timeout);
            for id in dead {
                println!(
                    "worker {id} declared DEAD (no heartbeat for {:?})",
                    self.timeout
                );
            }
        }
    }
}
