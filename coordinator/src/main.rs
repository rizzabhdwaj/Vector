mod api_server;
mod grpc_server;
mod task_queue;
mod worker;
use crate::{scheduler::schedule_task::Scheduler, worker::workers::Workforce};
mod scheduler;
use crate::task_queue::queue::TaskQueue;
use std::sync::{Arc, Mutex};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //both Scheduler and Api gateway will have a shared access to these 2 variable's through ArcMutex
    let work_force = Arc::new(Mutex::new(Workforce::init_workforce()));
    let task_q = Arc::new(Mutex::new(TaskQueue::create_queue()));

    let _scheduler = Arc::new(Mutex::new(Scheduler::init_scheduler(
        Arc::clone(&task_q),
        Arc::clone(&work_force),
    )));

    tokio::select! {
        result = api_server::api_gateway(Arc::clone(&work_force), Arc::clone(&task_q)) => result,
        result = grpc_server::grpc_server(Arc::clone(&work_force)) => {
            result.map_err(std::io::Error::other)
        }
    }
}
