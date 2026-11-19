use crate::grpc_server::heartbeat::HeartbeatRequest;

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct Worker {
    pub(crate) id: i32,
    pub(crate) available: bool,
}

// we need a way to keep limited workers and update them as per heartbeat
#[derive(Default, serde::Serialize)]
pub struct Workforce {
    pub(crate) workers: Vec<Worker>,
}

impl Workforce {
    pub fn update_worker(&mut self, heartbeat: HeartbeatRequest) {
        for i in &self.workers {
            if i.id == heartbeat.worker_id {
                // The scheduler owns availability after registration.
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
