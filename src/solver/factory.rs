use crate::domain::{
    models::OptimizationProblem,
    solver_service::SolverService,
    value_objects::SolverBackend,
};
use crate::solver::{CoinCbcSolver, HighsSolver};
use std::sync::Arc;

/// Factory for creating solver instances based on configuration
pub struct SolverFactory;

impl SolverFactory {
    /// Create a solver based on the problem configuration
    pub fn create_solver(problem: &OptimizationProblem) -> Arc<dyn SolverService> {
        Self::create_from_backend(problem.solver_config.backend, problem.is_mixed_integer())
    }

    /// Create a solver for a specific backend
    pub fn create_from_backend(backend: SolverBackend, _is_mip: bool) -> Arc<dyn SolverService> {
        match backend {
            SolverBackend::Auto => Arc::new(HighsSolver::new()),
            SolverBackend::CoinCbc => Arc::new(CoinCbcSolver::new()),
            SolverBackend::Highs => Arc::new(HighsSolver::new()),
        }
    }

    /// Get the default solver (HiGHS)
    pub fn default_solver() -> Arc<dyn SolverService> {
        Arc::new(HighsSolver::new())
    }
}
