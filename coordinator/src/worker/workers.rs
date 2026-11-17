use crate::grpc_server::heartbeat::HeartbeatRequest;

#[derive(Clone, Debug, serde::Serialize)]
struct Worker {
    id: i32,
    available: bool,
}
#[derive(Default, serde::Serialize)]
pub struct Workforce {
    workers: Vec<Worker>,
}

impl Workforce {
    pub fn new_worker(&mut self, heartbeat: HeartbeatRequest) {
        self.workers.push(Worker {
            id: heartbeat.worker_id,
            available: heartbeat.available,
        });
    }

    pub fn init_workforce() -> Workforce {
        Workforce::default()
    }
}
