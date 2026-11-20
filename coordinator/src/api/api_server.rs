use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, get, post, web};
use std::sync::{Arc, Mutex};

use crate::task_queue::queue::{Task, TaskQueue};
use crate::worker::workers::Workforce;
const PORT: u16 = 3030;
#[get("/metrics")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().json("fine ! ")
}

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
        Ok(queue) => HttpResponse::Ok().json(&queue.outputs),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/tasks")]
async fn create_task(queue: web::Data<Mutex<TaskQueue>>, body: web::Json<Task>) -> HttpResponse {
    match queue.lock() {
        Ok(mut queue) => {
            queue.new_task(body.into_inner());
            HttpResponse::Created().finish()
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn api_gateway(
    workforce: Arc<Mutex<Workforce>>,
    task_queue: Arc<Mutex<TaskQueue>>,
) -> std::io::Result<()> {
    let workforce = web::Data::from(workforce);
    let task_q = web::Data::from(task_queue);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(workforce.clone())
            .app_data(task_q.clone())
            .wrap(Cors::permissive())
            .service(health)
            .service(get_workforce)
            .service(create_task)
            .service(get_tasks)
            .service(get_outputs)
    })
    .bind(("0.0.0.0", PORT))?;

    println!("HTTP server running on {}", PORT);
    server.run().await
}
