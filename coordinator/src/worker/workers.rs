use crate::grpc_server::heartbeat::HeartbeatRequest;
use std::time::Duration;

#[derive(Clone, Debug, serde::Serialize)]
struct Worker {
    id: i32,
    available: bool,
}

// we need a way to keep limited workers and update them as per heartbeat
#[derive(Default, serde::Serialize)]
pub struct Workforce {
    workers: Vec<Worker>,
}

impl Workforce {
    pub fn update_worker(&mut self, heartbeat: HeartbeatRequest) {
        for i in &mut self.workers {
            if i.id == heartbeat.worker_id {
                // only update the availability
                i.available = heartbeat.available;
                return;
            }
        }
        self.workers.push(Worker {
            id: heartbeat.worker_id,
            available: heartbeat.available,
        });
    }

    pub fn init_workforce() -> Workforce {
        Workforce::default()
    }
}
