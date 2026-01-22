// Domain value objects representing core business concepts

use std::fmt;

/// Type of decision variable in the optimization problem
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableType {
    /// Continuous real number (x ∈ ℝ)
    Continuous,
    /// Integer number (x ∈ ℤ)
    Integer,
    /// Binary variable (x ∈ {0, 1})
    Binary,
}

/// Type of constraint comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintType {
    /// Less than or equal (≤)
    LessThanOrEqual,
    /// Equal (=)
    Equal,
    /// Greater than or equal (≥)
    GreaterThanOrEqual,
}

/// Direction of optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationType {
    /// Minimize the objective function
    Minimize,
    /// Maximize the objective function
    Maximize,
}

/// Status of the optimization solution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolutionStatus {
    /// Found optimal solution
    Optimal,
    /// Found feasible solution (may not be optimal)
    Feasible,
    /// Problem has no feasible solution
    Infeasible,
    /// Objective can be improved infinitely
    Unbounded,
    /// Time limit reached
    TimeLimit,
    /// Iteration limit reached
    IterationLimit,
    /// Node limit reached (MIP)
    NodeLimit,
    /// Solver error occurred
    Error,
    /// Solve interrupted by user
    Interrupted,
}

impl fmt::Display for SolutionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolutionStatus::Optimal => write!(f, "Optimal"),
            SolutionStatus::Feasible => write!(f, "Feasible"),
            SolutionStatus::Infeasible => write!(f, "Infeasible"),
            SolutionStatus::Unbounded => write!(f, "Unbounded"),
            SolutionStatus::TimeLimit => write!(f, "Time Limit Reached"),
            SolutionStatus::IterationLimit => write!(f, "Iteration Limit Reached"),
            SolutionStatus::NodeLimit => write!(f, "Node Limit Reached"),
            SolutionStatus::Error => write!(f, "Error"),
            SolutionStatus::Interrupted => write!(f, "Interrupted"),
        }
    }
}

/// Solver backend to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverBackend {
    /// Automatically select best solver
    Auto,
    /// COIN-OR CBC solver
    CoinCbc,
}

impl fmt::Display for SolverBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverBackend::Auto => write!(f, "Auto"),
            SolverBackend::CoinCbc => write!(f, "COIN-OR CBC"),
        }
    }
}
