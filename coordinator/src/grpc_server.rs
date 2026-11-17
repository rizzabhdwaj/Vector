use tonic::{Request, Response, Status, transport::Server};

pub mod heartbeat {
    tonic::include_proto!("heartbeat");
}

use heartbeat::{
    HeartbeatRequest,
    heartbeat_server::{Heartbeat, HeartbeatServer},
};

struct HeartbeatService;

#[tonic::async_trait]
impl Heartbeat for HeartbeatService {
    async fn send_heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<()>, Status> {
        println!("Heartbeat received: {:?}", request.into_inner());
        Ok(Response::new(()))
    }
}

pub async fn grpc_server() -> Result<(), tonic::transport::Error> {
    Server::builder()
        .add_service(HeartbeatServer::new(HeartbeatService))
        .serve(([0, 0, 0, 0], 50052).into())
        .await
}
