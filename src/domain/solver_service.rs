// Domain service interface for solving optimization problems
// Defines the contract that any solver implementation must follow (Dependency Inversion Principle)

use super::models::{OptimizationProblem, Solution};

/// Error types for the solver service
#[derive(Debug, thiserror::Error)]
pub enum SolverError {
    #[error("Invalid problem: {0}")]
    InvalidProblem(String),

    #[error("Solver not available: {0}")]
    SolverNotAvailable(String),

    #[error("Solver execution failed: {0}")]
    ExecutionFailed(String),
}

pub type Result<T> = std::result::Result<T, SolverError>;

/// Domain service interface for optimization solvers
///
/// This trait defines the contract that all solver implementations must follow.
/// It allows us to swap solver backends without changing business logic (Open/Closed Principle).
pub trait SolverService: Send + Sync {
    /// Solve an optimization problem
    fn solve(&self, problem: &OptimizationProblem) -> Result<Solution>;

    /// Validate a problem without solving it
    fn validate(&self, problem: &OptimizationProblem) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        // Check objective has coefficients
        if problem.objective.coefficients.is_empty() {
            errors.push("Objective must have at least one coefficient".to_string());
        }

        let num_vars = problem.num_variables();

        // Check variables match objective
        if !problem.variables.is_empty() && problem.variables.len() != num_vars {
            errors.push(format!(
                "Number of variables ({}) doesn't match objective coefficients ({})",
                problem.variables.len(),
                num_vars
            ));
        }

        // Check constraints
        for (i, constraint) in problem.constraints.iter().enumerate() {
            if constraint.num_variables() != num_vars {
                errors.push(format!(
                    "Constraint {} has {} coefficients but problem has {} variables",
                    i,
                    constraint.num_variables(),
                    num_vars
                ));
            }
        }

        // Check variable bounds
        for (i, var) in problem.variables.iter().enumerate() {
            if let Some(upper) = var.upper_bound {
                if var.lower_bound > upper {
                    errors.push(format!(
                        "Variable {} '{}' has lower bound ({}) > upper bound ({})",
                        i, var.name, var.lower_bound, upper
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(Vec::new())
        } else {
            Err(SolverError::InvalidProblem(errors.join("; ")))
        }
    }

    /// Get the name of this solver backend
    fn name(&self) -> &str;

    /// Check if this solver supports mixed-integer programming
    fn supports_mip(&self) -> bool;
}
