use tonic::{Request, Response, Status, transport::Server};

pub mod heartbeat {
    tonic::include_proto!("heartbeat");
}

use heartbeat::{
    HeartbeatRequest,
    heartbeat_server::{Heartbeat, HeartbeatServer},
};
use std::sync::{Arc, Mutex};

use crate::worker::workers::Workforce;
struct HeartbeatService {
    workforce: Arc<Mutex<Workforce>>,
}

#[tonic::async_trait]
impl Heartbeat for HeartbeatService {
    async fn send_heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<()>, Status> {
        let heartbeat = request.into_inner();
        println!("Heartbeat received: {:?}", heartbeat);

        self.workforce
            .lock()
            .map_err(|_| Status::internal("Workforce lock poisoned"))?
            .update_worker(heartbeat);

        Ok(Response::new(()))
    }
}

pub async fn grpc_server(workforce: Arc<Mutex<Workforce>>) -> Result<(), tonic::transport::Error> {
    let service = HeartbeatService { workforce };

    Server::builder()
        .add_service(HeartbeatServer::new(service))
        .serve(([0, 0, 0, 0], 50052).into())
        .await
}
