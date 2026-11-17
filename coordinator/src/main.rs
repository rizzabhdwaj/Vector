mod api_server;
mod grpc_server;
mod worker;
use std::sync::{Arc, Mutex};
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let work_force = Arc::new(Mutex::new(worker::workers::Workforce::init_workforce()));
    tokio::select! {
        result = api_server::api_gateway(Arc::clone(&work_force)) => result,
        result = grpc_server::grpc_server(Arc::clone(&work_force)) => {
            result.map_err(std::io::Error::other)
        }
    }
}
