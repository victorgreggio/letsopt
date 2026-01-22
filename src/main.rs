use letsopt::{start_server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse address
    let address = "0.0.0.0:50051".parse()?;

    // Configure and start server
    let config = ServerConfig::new(address);
    start_server(config).await?;

    Ok(())
}
