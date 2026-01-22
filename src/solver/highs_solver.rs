// HiGHS Solver Adapter
// Implements the SolverService interface for HiGHS
// This is an adapter pattern - translates our domain models to HiGHS API

use crate::domain::{
    models::{OptimizationProblem, Solution as DomainSolution, SolverStatistics},
    solver_service::{Result, SolverError, SolverService},
    value_objects::{
        ConstraintType, OptimizationType, SolutionStatus as DomainSolutionStatus, VariableType,
    },
};
use std::time::Instant;

pub struct HighsSolver;

impl HighsSolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HighsSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl SolverService for HighsSolver {
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

        // Use HiGHS RowProblem (add variables first, then constraints)
        use highs::{HighsModelStatus, RowProblem, Sense};

        let mut pb = RowProblem::default();
        let mut vars = Vec::new();

        // Add variables
        for var_def in &problem.variables {
            let lower = var_def.lower_bound;
            let upper = var_def.upper_bound.unwrap_or(f64::INFINITY);
            
            let obj_coeff = problem.objective.coefficients.get(vars.len()).copied().unwrap_or(0.0);
            
            let col = match var_def.variable_type {
                VariableType::Integer | VariableType::Binary => {
                    pb.add_integer_column(obj_coeff, lower..upper)
                }
                VariableType::Continuous => {
                    pb.add_column(obj_coeff, lower..upper)
                }
            };
            vars.push(col);
        }

        // If no variables specified, create defaults
        if problem.variables.is_empty() {
            for &coeff in problem.objective.coefficients.iter() {
                let col = pb.add_column(coeff, 0..);
                vars.push(col);
            }
        }

        // Add constraints
        for constraint in &problem.constraints {
            let mut terms = Vec::new();
            for (i, &coeff) in constraint.coefficients.iter().enumerate() {
                if coeff != 0.0 && i < vars.len() {
                    terms.push((vars[i], coeff));
                }
            }

            match constraint.constraint_type {
                ConstraintType::LessThanOrEqual => {
                    pb.add_row(..=constraint.bound, &terms);
                }
                ConstraintType::Equal => {
                    pb.add_row(constraint.bound..=constraint.bound, &terms);
                }
                ConstraintType::GreaterThanOrEqual => {
                    pb.add_row(constraint.bound.., &terms);
                }
            }
        }

        // Solve the problem
        let sense = if problem.objective.optimization_type == OptimizationType::Maximize {
            Sense::Maximise
        } else {
            Sense::Minimise
        };

        let solved = pb.optimise(sense).solve();
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
        match solved.status() {
            HighsModelStatus::Optimal => {
                let solution_data = solved.get_solution();
                let variable_values = solution_data.columns().to_vec();
                
                // Calculate objective value
                let mut actual_obj = 0.0;
                for (i, &val) in variable_values.iter().enumerate() {
                    if let Some(&coeff) = problem.objective.coefficients.get(i) {
                        actual_obj += coeff * val;
                    }
                }

                let mut solution = DomainSolution::optimal(actual_obj, variable_values);
                solution.statistics = statistics;
                solution.message = format!("Optimal solution found for '{}'", problem.name);

                Ok(solution)
            }
            HighsModelStatus::Infeasible => {
                let mut solution = DomainSolution::new(
                    DomainSolutionStatus::Infeasible,
                    "Problem is infeasible: no solution satisfies all constraints",
                );
                solution.statistics = statistics;
                Ok(solution)
            }
            HighsModelStatus::Unbounded | HighsModelStatus::UnboundedOrInfeasible => {
                let mut solution = DomainSolution::new(
                    DomainSolutionStatus::Unbounded,
                    "Problem is unbounded: objective can be improved infinitely",
                );
                solution.statistics = statistics;
                Ok(solution)
            }
            status => Err(SolverError::ExecutionFailed(format!(
                "HiGHS solver returned status: {:?}",
                status
            ))),
        }
    }

    fn name(&self) -> &str {
        "HiGHS"
    }

    fn supports_mip(&self) -> bool {
        true
    }
}

