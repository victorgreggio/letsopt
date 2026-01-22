use crate::domain::{
    models::{OptimizationProblem, Solution as DomainSolution, SolverStatistics},
    solver_service::{Result, SolverError, SolverService},
    value_objects::{
        ConstraintType, OptimizationType, SolutionStatus as DomainSolutionStatus, VariableType,
    },
};
use good_lp::{
    solvers::coin_cbc, variable, variables, Expression, ResolutionError,
    Solution as GoodLpSolutionTrait, SolverModel, Variable as GoodLpVariable,
};
use std::time::Instant;

pub struct CoinCbcSolver;

impl CoinCbcSolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CoinCbcSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl SolverService for CoinCbcSolver {
    fn solve(&self, problem: &OptimizationProblem) -> Result<DomainSolution> {
        // Validate first
        self.validate(problem)?;

        let start_time = Instant::now();
        let num_vars = problem.num_variables();

        // Count integer variables
        let num_integer = problem
            .variables
            .iter()
            .filter(|v| matches!(v.variable_type, VariableType::Integer))
            .count() as u32;
        let num_binary = problem
            .variables
            .iter()
            .filter(|v| matches!(v.variable_type, VariableType::Binary))
            .count() as u32;

        // Build variables using good_lp
        let mut vars = variables!();
        let mut lp_variables: Vec<GoodLpVariable> = Vec::new();

        for var_def in problem.variables.iter() {
            let lower = var_def.lower_bound;
            let upper = var_def.upper_bound.unwrap_or(f64::INFINITY);

            let var = match var_def.variable_type {
                VariableType::Binary | VariableType::Integer => {
                    vars.add(variable().integer().min(lower).max(upper))
                }
                VariableType::Continuous => vars.add(variable().min(lower).max(upper)),
            };
            lp_variables.push(var);
        }

        // If no variables specified, create defaults
        if problem.variables.is_empty() {
            for _ in 0..num_vars {
                let var = vars.add(variable().min(0.0));
                lp_variables.push(var);
            }
        }

        // Build objective expression
        let is_maximize = problem.objective.optimization_type == OptimizationType::Maximize;
        let mut obj_expr: Expression = 0.into();

        for (i, &coeff) in problem.objective.coefficients.iter().enumerate() {
            if coeff != 0.0 {
                // good_lp minimizes, so negate for maximization
                let c = if is_maximize { -coeff } else { coeff };
                obj_expr += c * lp_variables[i];
            }
        }

        // Build constraints
        let mut lp_model = vars.minimise(obj_expr).using(coin_cbc::coin_cbc);

        for constraint in &problem.constraints {
            let mut lhs: Expression = 0.into();
            for (i, &coeff) in constraint.coefficients.iter().enumerate() {
                if coeff != 0.0 {
                    lhs += coeff * lp_variables[i];
                }
            }

            match constraint.constraint_type {
                ConstraintType::LessThanOrEqual => {
                    lp_model = lp_model.with(lhs.leq(constraint.bound));
                }
                ConstraintType::Equal => {
                    lp_model = lp_model.with(lhs.eq(constraint.bound));
                }
                ConstraintType::GreaterThanOrEqual => {
                    lp_model = lp_model.with(lhs.geq(constraint.bound));
                }
            }
        }

        // Solve the problem
        let solution_result = lp_model.solve();
        let solve_time = start_time.elapsed().as_secs_f64() * 1000.0;

        // Build statistics
        let statistics = SolverStatistics {
            simplex_iterations: 0,
            nodes_explored: 0,
            solve_time_ms: solve_time,
            num_variables: num_vars as u32,
            num_constraints: problem.constraints.len() as u32,
            num_integer_vars: num_integer,
            num_binary_vars: num_binary,
        };

        // Process result
        match solution_result {
            Ok(sol) => {
                // Extract variable values
                let mut variable_values = vec![0.0; num_vars];
                for (i, &var) in lp_variables.iter().enumerate() {
                    variable_values[i] = sol.value(var);
                }

                // Calculate actual objective value
                let mut actual_obj = 0.0;
                for (i, &coeff) in problem.objective.coefficients.iter().enumerate() {
                    actual_obj += coeff * variable_values[i];
                }

                let mut solution = DomainSolution::optimal(actual_obj, variable_values);
                solution.statistics = statistics;
                solution.message = format!("Optimal solution found for '{}'", problem.name);

                Ok(solution)
            }
            Err(ResolutionError::Infeasible) => {
                let mut solution = DomainSolution::new(
                    DomainSolutionStatus::Infeasible,
                    "Problem is infeasible: no solution satisfies all constraints",
                );
                solution.statistics = statistics;
                Ok(solution)
            }
            Err(ResolutionError::Unbounded) => {
                let mut solution = DomainSolution::new(
                    DomainSolutionStatus::Unbounded,
                    "Problem is unbounded: objective can be improved infinitely",
                );
                solution.statistics = statistics;
                Ok(solution)
            }
            Err(e) => Err(SolverError::ExecutionFailed(format!("{:?}", e))),
        }
    }

    fn name(&self) -> &str {
        "COIN-OR CBC"
    }

    fn supports_mip(&self) -> bool {
        true
    }
}
