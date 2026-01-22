use super::value_objects::{
    ConstraintType, OptimizationType, SolutionStatus, SolverBackend, VariableType,
};

/// Decision variable in an optimization problem
#[derive(Debug, Clone)]
pub struct Variable {
    pub variable_type: VariableType,
    pub lower_bound: f64,
    pub upper_bound: Option<f64>,
    pub name: String,
}

impl Variable {
    pub fn continuous(name: impl Into<String>) -> Self {
        Self {
            variable_type: VariableType::Continuous,
            lower_bound: 0.0,
            upper_bound: None,
            name: name.into(),
        }
    }

    pub fn integer(name: impl Into<String>) -> Self {
        Self {
            variable_type: VariableType::Integer,
            lower_bound: 0.0,
            upper_bound: None,
            name: name.into(),
        }
    }

    pub fn binary(name: impl Into<String>) -> Self {
        Self {
            variable_type: VariableType::Binary,
            lower_bound: 0.0,
            upper_bound: Some(1.0),
            name: name.into(),
        }
    }

    pub fn with_bounds(mut self, lower: f64, upper: Option<f64>) -> Self {
        self.lower_bound = lower;
        self.upper_bound = upper;
        self
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self.variable_type,
            VariableType::Integer | VariableType::Binary
        )
    }
}

/// Objective function to minimize or maximize
#[derive(Debug, Clone)]
pub struct ObjectiveFunction {
    pub optimization_type: OptimizationType,
    pub coefficients: Vec<f64>,
    pub variable_names: Vec<String>,
}

impl ObjectiveFunction {
    pub fn new(optimization_type: OptimizationType, coefficients: Vec<f64>) -> Self {
        let variable_names = (0..coefficients.len()).map(|i| format!("x{}", i)).collect();

        Self {
            optimization_type,
            coefficients,
            variable_names,
        }
    }

    pub fn with_names(mut self, names: Vec<String>) -> Self {
        self.variable_names = names;
        self
    }

    pub fn num_variables(&self) -> usize {
        self.coefficients.len()
    }
}

/// Linear constraint on variables
#[derive(Debug, Clone)]
pub struct Constraint {
    pub constraint_type: ConstraintType,
    pub coefficients: Vec<f64>,
    pub bound: f64,
    pub name: String,
}

impl Constraint {
    pub fn new(constraint_type: ConstraintType, coefficients: Vec<f64>, bound: f64) -> Self {
        Self {
            constraint_type,
            coefficients,
            bound,
            name: String::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn num_variables(&self) -> usize {
        self.coefficients.len()
    }
}

/// Configuration for the solver
#[derive(Debug, Clone)]
pub struct SolverConfig {
    pub backend: SolverBackend,
    pub time_limit: Option<f64>,
    pub gap_tolerance: Option<f64>,
    pub verbose: bool,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            backend: SolverBackend::Auto,
            time_limit: None,
            gap_tolerance: None,
            verbose: false,
        }
    }
}

/// Complete optimization problem
#[derive(Debug, Clone)]
pub struct OptimizationProblem {
    pub name: String,
    pub description: String,
    pub objective: ObjectiveFunction,
    pub constraints: Vec<Constraint>,
    pub variables: Vec<Variable>,
    pub solver_config: SolverConfig,
}

impl OptimizationProblem {
    pub fn new(objective: ObjectiveFunction) -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            objective,
            constraints: Vec::new(),
            variables: Vec::new(),
            solver_config: SolverConfig::default(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn add_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn with_variables(mut self, variables: Vec<Variable>) -> Self {
        self.variables = variables;
        self
    }

    pub fn with_config(mut self, config: SolverConfig) -> Self {
        self.solver_config = config;
        self
    }

    pub fn num_variables(&self) -> usize {
        self.objective.num_variables()
    }

    pub fn num_integer_variables(&self) -> usize {
        self.variables.iter().filter(|v| v.is_integer()).count()
    }

    pub fn is_mixed_integer(&self) -> bool {
        self.num_integer_variables() > 0
    }
}

/// Statistics about the solve process
#[derive(Debug, Clone, Default)]
pub struct SolverStatistics {
    pub simplex_iterations: u64,
    pub nodes_explored: u64,
    pub solve_time_ms: f64,
    pub num_variables: u32,
    pub num_constraints: u32,
    pub num_integer_vars: u32,
    pub num_binary_vars: u32,
}

/// Quality metrics for the solution
#[derive(Debug, Clone, Default)]
pub struct SolutionQuality {
    pub max_constraint_violation: f64,
    pub max_integrality_violation: f64,
    pub reliability: f64,
}

/// Solution to an optimization problem
#[derive(Debug, Clone)]
pub struct Solution {
    pub status: SolutionStatus,
    pub optimal_value: Option<f64>,
    pub best_bound: Option<f64>,
    pub gap: Option<f64>,
    pub variable_values: Vec<f64>,
    pub dual_values: Vec<f64>,
    pub message: String,
    pub statistics: SolverStatistics,
    pub quality: SolutionQuality,
}

impl Solution {
    pub fn new(status: SolutionStatus, message: impl Into<String>) -> Self {
        Self {
            status,
            optimal_value: None,
            best_bound: None,
            gap: None,
            variable_values: Vec::new(),
            dual_values: Vec::new(),
            message: message.into(),
            statistics: SolverStatistics::default(),
            quality: SolutionQuality::default(),
        }
    }

    pub fn optimal(value: f64, variable_values: Vec<f64>) -> Self {
        Self {
            status: SolutionStatus::Optimal,
            optimal_value: Some(value),
            best_bound: Some(value),
            gap: Some(0.0),
            variable_values,
            dual_values: Vec::new(),
            message: "Optimal solution found".to_string(),
            statistics: SolverStatistics::default(),
            quality: SolutionQuality::default(),
        }
    }

    pub fn with_statistics(mut self, statistics: SolverStatistics) -> Self {
        self.statistics = statistics;
        self
    }

    pub fn with_quality(mut self, quality: SolutionQuality) -> Self {
        self.quality = quality;
        self
    }

    pub fn is_optimal(&self) -> bool {
        self.status == SolutionStatus::Optimal
    }

    pub fn is_feasible(&self) -> bool {
        matches!(
            self.status,
            SolutionStatus::Optimal | SolutionStatus::Feasible
        )
    }
}
