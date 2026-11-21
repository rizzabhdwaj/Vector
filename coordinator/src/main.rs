mod api;
mod task_queue;
mod worker;
use std::time::Duration;

use crate::api::{api_server, db, grpc_server};
use crate::{scheduler::schedule_task::Scheduler, worker::reaper, worker::workers::Workforce};
mod scheduler;
use crate::task_queue::queue::TaskQueue;
use std::sync::{Arc, Mutex};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //both Scheduler and Api gateway will have a shared access to these 2 variable's through ArcMutex

    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("database url not found");
    let pool = db::init_pool(&database_url)
        .await
        .expect("db connection failed");
    let work_force = Arc::new(Mutex::new(Workforce::init_workforce()));
    let task_q = Arc::new(Mutex::new(TaskQueue::create_queue()));

    let scheduler = Scheduler::init_scheduler(Arc::clone(&task_q), Arc::clone(&work_force));
    let reaper = reaper::Reaper::new(Arc::clone(&work_force), Duration::from_secs(15));
    tokio::select! {
        _ = scheduler.run() => Ok(()),
        _ = reaper.run() => Ok(()),
        result = api_server::api_gateway(Arc::clone(&work_force), Arc::clone(&task_q)) => result,
        result = grpc_server::grpc_server(Arc::clone(&work_force)) => {
            result.map_err(std::io::Error::other)
        }
    }
}
