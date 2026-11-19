mod api_server;
mod grpc_server;
mod task_queue;
mod worker;
use crate::task_queue::queue::{Task, TaskQueue};
use std::sync::{Arc, Mutex};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let work_force = Arc::new(Mutex::new(worker::workers::Workforce::init_workforce()));
    let task_q = Arc::new(Mutex::new(TaskQueue::create_queue()));

    tokio::select! {
        result = api_server::api_gateway(Arc::clone(&work_force), Arc::clone(&task_q)) => result,
        result = grpc_server::grpc_server(Arc::clone(&work_force)) => {
            result.map_err(std::io::Error::other)
        }
    }
}
