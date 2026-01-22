// Infrastructure: Server setup and configuration
// Single Responsibility: Manage server lifecycle and configuration

use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;

use crate::application::mappers::lp_solver::linear_programming_solver_server::LinearProgrammingSolverServer;
use crate::application::GrpcLpSolverService;
use crate::domain::solver_service::SolverService;

pub struct ServerConfig {
    pub address: SocketAddr,
    pub solver: Arc<dyn SolverService>,
}

impl ServerConfig {
    pub fn new(address: SocketAddr, solver: Arc<dyn SolverService>) -> Self {
        Self { address, solver }
    }
}

pub async fn start_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let service = GrpcLpSolverService::new(config.solver);

    print_banner(&config.address);

    Server::builder()
        .add_service(LinearProgrammingSolverServer::new(service))
        .serve(config.address)
        .await?;

    Ok(())
}

fn print_banner(address: &SocketAddr) {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  LetsOpt - Linear & Mixed-Integer Programming Solver      ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  Powered by: COIN-OR CBC                                  ║");
    println!("║  Listening on: {:42} ║", address);
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  Supported Features:                                      ║");
    println!("║    ✓ Linear Programming (LP)                              ║");
    println!("║    ✓ Mixed-Integer Programming (MIP)                      ║");
    println!("║    ✓ Binary Variables                                     ║");
    println!("║    ✓ Integer Variables                                    ║");
    println!("║    ✓ Branch-and-Cut Algorithm                             ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("\n🚀 Ready to solve optimization problems!\n");
}
