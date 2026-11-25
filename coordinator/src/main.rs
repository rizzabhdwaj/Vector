mod api;
mod db_worker;
mod rescheduler {
    pub mod manager;
}
mod scheduler;
mod task_queue;
mod worker;
mod ws_server;
use std::time::Duration;

use crate::api::metrics::Metrics;
use crate::api::{api_server, db, grpc_server};
use crate::db_worker::daemon;
use crate::rescheduler::manager::orphan_mgr;
use crate::scheduler::schedule_task::Scheduler;
use crate::task_queue::queue::TaskQueue;
use crate::worker::reaper::Reaper;
use crate::worker::workers::Workforce;
use std::sync::{Arc, Mutex};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("database url not found");
    let pool = db::init_pool(&database_url)
        .await
        .expect("db connection failed");

    let work_force = Arc::new(Mutex::new(Workforce::init_workforce()));
    let task_q = Arc::new(Mutex::new(TaskQueue::create_queue()));
    let metrics = Arc::new(Metrics::new());

    let (persist_tx, persist_rx) = tokio::sync::mpsc::channel(16);
    let persister = daemon::Persister::new(persist_rx, pool.clone());

    let scheduler = Scheduler::init_scheduler(
        Arc::clone(&task_q),
        Arc::clone(&work_force),
        Some(persist_tx),
        Arc::clone(&metrics),
    );

    let reaper = Reaper::new(
        Arc::clone(&work_force),
        Duration::from_secs(15),
        Arc::clone(&metrics),
    );

    let orphan_manager = orphan_mgr::new(Arc::clone(&work_force), Arc::clone(&task_q));

    tokio::spawn(scheduler.run());
    tokio::spawn(reaper.run());
    tokio::spawn(async move {
        if let Err(err) = persister.run().await {
            eprintln!("persister stopped: {err}");
        }
    });
    tokio::spawn(orphan_manager.run());

    tokio::select! {
        result = api_server::api_gateway(Arc::clone(&work_force), Arc::clone(&task_q), Arc::clone(&metrics), pool) => result,
        result = grpc_server::grpc_server(Arc::clone(&work_force), Arc::clone(&metrics)) => {
            result.map_err(std::io::Error::other)
        }
        result = ws_server::start_ws_server(Arc::clone(&work_force), Arc::clone(&task_q)) => result,
    }
}
