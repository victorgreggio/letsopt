use letsopt::{start_server, CoinCbcSolver, ServerConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse address
    let address = "0.0.0.0:50051".parse()?;

    // Create solver instance
    let solver = Arc::new(CoinCbcSolver::new());

    // Configure and start server
    let config = ServerConfig::new(address, solver);
    start_server(config).await?;

    Ok(())
}
