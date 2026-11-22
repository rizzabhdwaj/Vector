use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, get, post, web};
use sqlx::PgPool;
use std::sync::{Arc, Mutex};

use crate::api::metrics::Metrics;
use crate::task_queue::queue::{Task, TaskQueue};
use crate::worker::workers::Workforce;
const PORT: u16 = 3030;

#[get("/workforce")]
async fn get_workforce(workforce: web::Data<Mutex<Workforce>>) -> HttpResponse {
    match workforce.lock() {
        Ok(workforce) => HttpResponse::Ok().json(&*workforce),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/tasks")]
async fn get_tasks(queue: web::Data<Mutex<TaskQueue>>) -> HttpResponse {
    match queue.lock() {
        Ok(queue) => HttpResponse::Ok().json(&*queue),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/outputs")]
async fn get_outputs(queue: web::Data<Mutex<TaskQueue>>) -> HttpResponse {
    match queue.lock() {
        Ok(mut queue) => HttpResponse::Ok().json((&mut *queue).take_outputs()),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/tasks")]
async fn create_task(
    queue: web::Data<Mutex<TaskQueue>>,
    metrics: web::Data<Metrics>,
    pool: web::Data<PgPool>,
    body: web::Json<Task>,
) -> HttpResponse {
    let task = body.into_inner();

    if let Err(err) = sqlx::query(
        "INSERT INTO tasks_pending (id, command, args, status) VALUES ($1, $2, $3, 'queued')
         ON CONFLICT (id) DO UPDATE SET command = EXCLUDED.command, args = EXCLUDED.args, status = 'queued'",
    )
    .bind(task.id as i64)
    .bind(&task.command)
    .bind(&task.args)
    .execute(pool.get_ref())
    .await
    {
        eprintln!("failed to persist task {}: {err}", task.id);
        return HttpResponse::InternalServerError().finish();
    }

    match queue.lock() {
        Ok(mut queue) => {
            queue.new_task(task);
            metrics.tasks_created.inc();
            HttpResponse::Created().finish()
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn api_gateway(
    workforce: Arc<Mutex<Workforce>>,
    task_queue: Arc<Mutex<TaskQueue>>,
    metrics: Arc<Metrics>,
    pool: PgPool,
) -> std::io::Result<()> {
    let workforce = web::Data::from(workforce);
    let task_q = web::Data::from(task_queue);
    let metrics = web::Data::from(metrics);
    let pool = web::Data::new(pool);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(workforce.clone())
            .app_data(task_q.clone())
            .app_data(metrics.clone())
            .app_data(pool.clone())
            .wrap(Cors::permissive())
            .configure(crate::api::metrics::configure)
            .service(get_workforce)
            .service(create_task)
            .service(get_tasks)
            .service(get_outputs)
    })
    .bind(("0.0.0.0", PORT))?;

    println!("HTTP server running on {}", PORT);
    server.run().await
}
