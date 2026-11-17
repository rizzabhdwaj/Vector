mod api_server;
mod grpc_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tokio::try_join!(api_server::api_gateway(), async {
        grpc_server::grpc_server().await.map_err(std::io::Error::other)
    })?;
    Ok(())
}
