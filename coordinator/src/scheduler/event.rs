use tokio::sync::{broadcast, mpsc};

#[derive(Clone, Debug, serde::Serialize)]
pub enum Event {
    TaskCreated {
        id: u32,
        command: String,
    },
    TaskRunning {
        id: u32,
        worker_id: i32,
    },
    TaskFinished {
        id: u32,
        worker_id: i32,
        output: String,
        success: bool,
    },
    WorkerDead {
        id: i32,
    },
}

#[derive(Clone)]
pub struct EventBus {
    persist_tx: mpsc::Sender<Event>,
    live_tx: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new() -> (Self, mpsc::Receiver<Event>) {
        let (persist_tx, persist_rx) = mpsc::channel(1024);
        let (live_tx, _) = broadcast::channel(256);
        (
            Self {
                persist_tx,
                live_tx,
            },
            persist_rx,
        )
    }

    pub async fn emit(&self, ev: Event) {
        let _ = self.live_tx.send(ev.clone()); // Err just means no subscribers
        let _ = self.persist_tx.send(ev).await; // bounded: gives backpressure
    }

    pub fn get_persist_sender(&self) -> mpsc::Sender<Event> {
        self.persist_tx.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.live_tx.subscribe()
    }
}