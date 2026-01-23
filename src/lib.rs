// Domain layer: Business logic and rules
pub mod domain;

// Application layer: Use cases and service orchestration
pub mod application;

// Infrastructure layer: External concerns (gRPC, server)
pub mod infrastructure;

// Solver adapters: Concrete implementations of SolverService
pub mod solver;

// Re-export commonly used types
pub use domain::{
    Constraint, ConstraintType, ObjectiveFunction, OptimizationProblem, OptimizationType, Solution,
    SolutionStatus, SolverError, SolverService, Variable, VariableType,
};

pub use application::GrpcLpSolverService;

pub use infrastructure::{start_server, ServerConfig};

pub use solver::{CoinCbcSolver, HighsSolver, SolverFactory};
