// Domain layer: Business logic and rules
pub mod domain;

// Application layer: Use cases and service orchestration
pub mod application;

// Infrastructure layer: External concerns (gRPC, server)
#[cfg(feature = "server")]
pub mod infrastructure;

// Solver adapters: Concrete implementations of SolverService
#[cfg(feature = "server")]
pub mod solver;

// Re-export commonly used types
pub use domain::{
    Constraint, ConstraintType, ObjectiveFunction, OptimizationProblem, OptimizationType, Solution,
    SolutionStatus, SolverError, SolverService, Variable, VariableType,
};

pub use application::GrpcLpSolverService;

#[cfg(feature = "server")]
pub use infrastructure::{start_server, ServerConfig};

#[cfg(feature = "server")]
pub use solver::{CoinCbcSolver, HighsSolver, SolverFactory};
