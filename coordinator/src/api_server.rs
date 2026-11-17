use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, get, web};
use std::sync::{Arc, Mutex};

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

pub async fn api_gateway(workforce: Arc<Mutex<Workforce>>) -> std::io::Result<()> {
    let workforce = web::Data::from(workforce);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(workforce.clone())
            .wrap(Cors::permissive())
            .service(health)
            .service(get_workforce)
    })
    .bind(("0.0.0.0", PORT))?;

    println!("HTTP server running on {}", PORT);
    server.run().await
}
