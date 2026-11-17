use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, get};

const PORT: u16 = 3030;

#[get("/metrics")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().json("fine ! ")
}

pub async fn api_gateway() -> std::io::Result<()> {
    let server = HttpServer::new(|| App::new().wrap(Cors::permissive()).service(health))
        .bind(("0.0.0.0", PORT))?;

    println!("HTTP server running on {}", PORT);
    server.run().await
}
