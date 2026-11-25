use std::sync::{Arc, Mutex};

use actix_web::{HttpResponse, Responder, web};
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

use crate::task_queue::queue::TaskQueue;
use crate::worker::workers::{WorkerStatus, Workforce};

/// Coordinator metrics exposed on `/metrics`.
///
/// Counters are updated by the components that own the event (API, scheduler,
/// heartbeat service, reaper), while gauges are refreshed from the current
/// in-memory state every time Prometheus scrapes the endpoint.
#[derive(Clone)]
pub struct Metrics {
    registry: Arc<Mutex<Registry>>,
    pub tasks_created: Counter,
    pub tasks_assigned: Counter,
    pub tasks_completed: Counter,
    pub tasks_failed: Counter,
    pub heartbeats_received: Counter,
    pub workers_reaped: Counter,
    pub workers_total: Gauge,
    pub workers_free: Gauge,
    pub workers_busy: Gauge,
    pub workers_dead: Gauge,
    pub tasks_queued: Gauge,
}

impl Metrics {
    pub fn new() -> Self {
        let mut registry = Registry::default();

        let tasks_created = Counter::default();
        let tasks_assigned = Counter::default();
        let tasks_completed = Counter::default();
        let tasks_failed = Counter::default();
        let heartbeats_received = Counter::default();
        let workers_reaped = Counter::default();
        let workers_total = Gauge::default();
        let workers_free = Gauge::default();
        let workers_busy = Gauge::default();
        let workers_dead = Gauge::default();
        let tasks_queued = Gauge::default();

        registry.register(
            "coordinator_tasks_created",
            "Total number of tasks accepted through the HTTP API",
            tasks_created.clone(),
        );
        registry.register(
            "coordinator_tasks_assigned",
            "Total number of tasks assigned to a worker",
            tasks_assigned.clone(),
        );
        registry.register(
            "coordinator_tasks_completed",
            "Total number of tasks completed successfully",
            tasks_completed.clone(),
        );
        registry.register(
            "coordinator_tasks_failed",
            "Total number of task dispatches or executions that failed",
            tasks_failed.clone(),
        );
        registry.register(
            "coordinator_heartbeats_received",
            "Total number of heartbeats received from workers",
            heartbeats_received.clone(),
        );
        registry.register(
            "coordinator_workers_reaped",
            "Total number of workers marked dead by the reaper",
            workers_reaped.clone(),
        );
        registry.register(
            "coordinator_workers_total",
            "Current number of registered workers",
            workers_total.clone(),
        );
        registry.register(
            "coordinator_workers_free",
            "Current number of workers available for work",
            workers_free.clone(),
        );
        registry.register(
            "coordinator_workers_busy",
            "Current number of workers currently running a task",
            workers_busy.clone(),
        );
        registry.register(
            "coordinator_workers_dead",
            "Current number of workers marked dead",
            workers_dead.clone(),
        );
        registry.register(
            "coordinator_tasks_queued",
            "Current number of tasks waiting in the queue",
            tasks_queued.clone(),
        );

        Self {
            registry: Arc::new(Mutex::new(registry)),
            tasks_created,
            tasks_assigned,
            tasks_completed,
            tasks_failed,
            heartbeats_received,
            workers_reaped,
            workers_total,
            workers_free,
            workers_busy,
            workers_dead,
            tasks_queued,
        }
    }

    /// Refresh gauges from the coordinator's in-memory state.
    pub fn update_gauges(&self, workforce: &Workforce, queue: &TaskQueue) {
        let mut free = 0;
        let mut busy = 0;
        let mut dead = 0;

        for worker in &workforce.workers {
            match worker.curr_status {
                WorkerStatus::Free => free += 1,
                WorkerStatus::Busy => busy += 1,
                WorkerStatus::Dead => dead += 1,
            }
        }

        self.workers_total.set(workforce.workers.len() as i64);
        self.workers_free.set(free);
        self.workers_busy.set(busy);
        self.workers_dead.set(dead);
        self.tasks_queued.set(queue.tasks().len() as i64);
    }

    /// Encode the registry using the Prometheus/OpenMetrics text format.
    pub fn encode(&self) -> Result<String, std::fmt::Error> {
        let registry = self.registry.lock().unwrap_or_else(|err| err.into_inner());
        let mut buffer = String::new();
        encode(&mut buffer, &registry)?;
        Ok(buffer)
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

async fn metrics_handler(
    metrics: web::Data<Metrics>,
    workforce: web::Data<Mutex<Workforce>>,
    task_queue: web::Data<Mutex<TaskQueue>>,
) -> impl Responder {
    if let Ok(workforce) = workforce.lock() {
        if let Ok(queue) = task_queue.lock() {
            metrics.update_gauges(&workforce, &queue);
        }
    }

    match metrics.encode() {
        Ok(body) => HttpResponse::Ok()
            .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
            .body(body),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Register the metrics route. Kept here so the route and its handler live in
/// the same module.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/metrics", web::get().to(metrics_handler))
        .route("/metric", web::get().to(metrics_handler));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};

    #[actix_web::test]
    async fn routes_export_counters_and_current_worker_gauges() {
        let metrics = web::Data::new(Metrics::new());
        metrics.tasks_completed.inc();
        let mut workforce = Workforce::init_workforce();
        workforce.update_worker(crate::api::grpc_server::heartbeat::HeartbeatRequest {
            worker_id: 5001,
            available: true,
        });
        let app = test::init_service(
            App::new()
                .app_data(metrics)
                .app_data(web::Data::new(Mutex::new(workforce)))
                .app_data(web::Data::new(Mutex::new(TaskQueue::create_queue())))
                .configure(configure),
        ).await;

        for path in ["/metric", "/metrics"] {
            let response = test::call_service(&app, test::TestRequest::get().uri(path).to_request()).await;
            assert!(response.status().is_success());
            let body = test::read_body(response).await;
            let body = std::str::from_utf8(&body).unwrap();
            assert!(body.contains("coordinator_tasks_completed_total 1"));
            assert!(body.contains("coordinator_workers_free 1"));
            assert!(body.contains("# EOF"));
        }
    }
}
