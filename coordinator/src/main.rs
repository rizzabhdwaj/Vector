mod api_server;
mod grpc_server;
mod worker;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _work_force = worker::workers::Workforce::init_workforce();
    tokio::try_join!(api_server::api_gateway(), async {
        grpc_server::grpc_server()
            .await
            .map_err(std::io::Error::other)
    })?;
    Ok(())
}
