use futures::{SinkExt as _, StreamExt as _};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tokio::{
    net::TcpListener,
    sync::mpsc,
    time::{Duration, interval, timeout},
};
use tokio_tungstenite::tungstenite::protocol::Message;

/// Message broadcast from the broadcaster task to all connected WebSocket clients.
/// Serialized as a flat object so the dashboard can match on `workers` / `tasks`.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum WsMessage {
    WorkerUpdated {
        workers: Vec<crate::worker::workers::Worker>,
    },
    TaskQueueUpdated {
        tasks: std::collections::VecDeque<crate::task_queue::queue::Task>,
        outputs: Vec<crate::task_queue::queue::Output>,
    },
}

/// Shared state managing connected WebSocket clients. Each client gets a
/// dedicated mpsc channel; the sender is stored here until the client leaves.
#[derive(Clone)]
pub struct WsState {
    senders: Arc<Mutex<Vec<mpsc::Sender<String>>>>,
}

impl WsState {
    /// Create a new WsState with an empty sender list.
    pub fn new() -> Self {
        WsState {
            senders: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a new client's sender to the list.
    pub fn add_sender(&self, sender: mpsc::Sender<String>) {
        let mut senders = self.senders.lock().unwrap();
        // `Sender` has no PartialEq; `same_channel` identifies senders of the
        // same channel instead.
        if !senders.iter().any(|s| s.same_channel(&sender)) {
            senders.push(sender);
        }
    }

    /// Remove a client's sender from the list.
    pub fn remove_sender(&self, sender: &mpsc::Sender<String>) {
        self.senders
            .lock()
            .unwrap()
            .retain(|s| !s.same_channel(sender));
    }

    /// Broadcast a serialized message to all connected clients.
    pub fn broadcast(&self, msg: &WsMessage) {
        let payload = serde_json::to_string(msg).expect("serialize websocket message");
        // Snapshots repeat on every tick, so a full client queue can skip this
        // update rather than delaying everyone else.
        self.senders.lock().unwrap().retain(|sender| {
            !matches!(
                sender.try_send(payload.clone()),
                Err(mpsc::error::TrySendError::Closed(_))
            )
        });
    }
}

pub async fn start_ws_server(
    workforce: Arc<Mutex<crate::worker::workers::Workforce>>,
    task_queue: Arc<Mutex<crate::task_queue::queue::TaskQueue>>,
) -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3031").await.map_err(|err| {
        std::io::Error::new(err.kind(), format!("WebSocket listen on port 3031: {err}"))
    })?;

    println!("WebSocket server listening on ws://127.0.0.1:3031");

    serve(listener, workforce, task_queue).await
}

async fn serve(
    listener: TcpListener,
    workforce: Arc<Mutex<crate::worker::workers::Workforce>>,
    task_queue: Arc<Mutex<crate::task_queue::queue::TaskQueue>>,
) -> std::io::Result<()> {
    let state = WsState::new();
    let mut broadcast_interval = interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, _) = result?;
                let client_state = state.clone();
                tokio::spawn(async move {
                    // A client that stalls during the handshake must not block other clients.
                    let Ok(Ok(ws_stream)) = timeout(Duration::from_secs(5), tokio_tungstenite::accept_async(stream)).await else {
                        return;
                    };
                    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                    let (client_sender, mut client_receiver) = mpsc::channel::<String>(32);
                    client_state.add_sender(client_sender.clone());

                    loop {
                        tokio::select! {
                            payload = client_receiver.recv() => {
                                let Some(payload) = payload else { break };
                                if !matches!(timeout(Duration::from_secs(5), ws_sender.send(Message::Text(payload.into()))).await, Ok(Ok(()))) {
                                    break;
                                }
                            }
                            incoming = ws_receiver.next() => {
                                match incoming {
                                    Some(Ok(Message::Close(_))) => {
                                        // Tungstenite queues a close reply when it reads this frame.
                                        let _ = timeout(Duration::from_secs(5), ws_sender.flush()).await;
                                        break;
                                    }
                                    Some(Err(_)) | None => break,
                                    // Incoming client messages are ignored.
                                    _ => {}
                                }
                            }
                        }
                    }

                    client_state.remove_sender(&client_sender);
                });
            }

            _ = broadcast_interval.tick() => {
                let workers = workforce.lock().unwrap().workers.clone();
                state.broadcast(&WsMessage::WorkerUpdated { workers });

                let msg = {
                    let queue = task_queue.lock().unwrap();
                    WsMessage::TaskQueueUpdated {
                        tasks: queue.tasks().clone(),
                        outputs: queue.outputs().to_vec(),
                    }
                };
                state.broadcast(&msg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::grpc_server::heartbeat::HeartbeatRequest;
    use crate::task_queue::queue::TaskQueue;
    use crate::worker::workers::Workforce;
    use serde_json::{Value, json};

    #[tokio::test]
    async fn sends_dashboard_snapshots_even_with_a_stalled_handshake() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut workforce = Workforce::init_workforce();
        workforce.update_worker(HeartbeatRequest {
            worker_id: 5001,
            available: true,
        });
        let server = tokio::spawn(serve(
            listener,
            Arc::new(Mutex::new(workforce)),
            Arc::new(Mutex::new(TaskQueue::create_queue())),
        ));

        let _stalled = tokio::net::TcpStream::connect(addr).await.unwrap();
        timeout(Duration::from_secs(2), async {
            let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/vector-ws"))
                .await
                .unwrap();
            let mut saw_workers = false;
            let mut saw_tasks = false;
            while !(saw_workers && saw_tasks) {
                let frame = ws.next().await.unwrap().unwrap();
                let data: Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
                if let Some(workers) = data.get("workers") {
                    assert_eq!(
                        *workers,
                        json!([{ "id": 5001, "curr_status": "free", "task_assigned": [] }])
                    );
                    saw_workers = true;
                } else {
                    assert_eq!(data, json!({ "tasks": [], "outputs": [] }));
                    saw_tasks = true;
                }
            }
            ws.close(None).await.unwrap();
            assert!(matches!(ws.next().await, Some(Ok(Message::Close(_)))));
        })
        .await
        .expect("timed out waiting for dashboard snapshots");

        server.abort();
    }
}
