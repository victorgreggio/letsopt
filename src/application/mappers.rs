// Mappers: Convert between gRPC protobuf types and domain models
// This keeps protobuf dependencies isolated from business logic (Dependency Inversion)

use crate::domain::{
    models::{
        Constraint, ObjectiveFunction, OptimizationProblem, Solution, SolverConfig, Variable,
    },
    value_objects::{
        ConstraintType, OptimizationType, SolutionStatus, SolverBackend, VariableType,
    },
};
use tonic::Status;

pub mod lp_solver {
    tonic::include_proto!("lp_solver");
}

use lp_solver as proto;

/// Convert protobuf Variable to domain Variable
pub fn proto_to_domain_variable(
    proto_var: &proto::Variable,
) -> std::result::Result<Variable, Box<Status>> {
    let variable_type = match proto::variable::VariableType::try_from(proto_var.r#type) {
        Ok(proto::variable::VariableType::Continuous) => VariableType::Continuous,
        Ok(proto::variable::VariableType::Integer) => VariableType::Integer,
        Ok(proto::variable::VariableType::Binary) => VariableType::Binary,
        Err(_) => return Err(Box::new(Status::invalid_argument("Invalid variable type"))),
    };

    Ok(Variable {
        variable_type,
        lower_bound: proto_var.lower_bound,
        upper_bound: proto_var.upper_bound,
        name: proto_var.name.clone(),
    })
}

/// Convert protobuf Constraint to domain Constraint
pub fn proto_to_domain_constraint(
    proto_constr: &proto::Constraint,
) -> std::result::Result<Constraint, Box<Status>> {
    let constraint_type = match proto::constraint::ConstraintType::try_from(proto_constr.r#type) {
        Ok(proto::constraint::ConstraintType::LessThanOrEqual) => ConstraintType::LessThanOrEqual,
        Ok(proto::constraint::ConstraintType::Equal) => ConstraintType::Equal,
        Ok(proto::constraint::ConstraintType::GreaterThanOrEqual) => {
            ConstraintType::GreaterThanOrEqual
        }
        Err(_) => {
            return Err(Box::new(Status::invalid_argument(
                "Invalid constraint type",
            )))
        }
    };

    Ok(Constraint {
        constraint_type,
        coefficients: proto_constr.coefficients.clone(),
        bound: proto_constr.bound,
        name: proto_constr.name.clone(),
    })
}

/// Convert protobuf ObjectiveFunction to domain ObjectiveFunction
pub fn proto_to_domain_objective(
    proto_obj: &proto::ObjectiveFunction,
) -> std::result::Result<ObjectiveFunction, Box<Status>> {
    let optimization_type =
        match proto::objective_function::OptimizationType::try_from(proto_obj.r#type) {
            Ok(proto::objective_function::OptimizationType::Minimize) => OptimizationType::Minimize,
            Ok(proto::objective_function::OptimizationType::Maximize) => OptimizationType::Maximize,
            Err(_) => {
                return Err(Box::new(Status::invalid_argument(
                    "Invalid optimization type",
                )))
            }
        };

    Ok(ObjectiveFunction {
        optimization_type,
        coefficients: proto_obj.coefficients.clone(),
        variable_names: proto_obj.variable_names.clone(),
    })
}

/// Convert protobuf OptimizationProblem to domain OptimizationProblem
pub fn proto_to_domain_problem(
    proto_prob: proto::OptimizationProblem,
) -> std::result::Result<OptimizationProblem, Box<Status>> {
    let objective = proto_prob
        .objective
        .ok_or_else(|| Box::new(Status::invalid_argument("Objective is required")))?;
    let objective = proto_to_domain_objective(&objective)?;

    // Create default variables if none provided
    let variables = if proto_prob.variables.is_empty() {
        // Create continuous non-negative variables by default
        (0..objective.num_variables())
            .map(|i| Variable::continuous(format!("x{}", i)))
            .collect()
    } else {
        proto_prob
            .variables
            .iter()
            .map(proto_to_domain_variable)
            .collect::<std::result::Result<Vec<_>, _>>()?
    };

    let constraints = proto_prob
        .constraints
        .iter()
        .map(proto_to_domain_constraint)
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let solver_config = if let Some(cfg) = proto_prob.solver_config {
        let backend = match proto::solver_config::SolverBackend::try_from(cfg.solver) {
            Ok(proto::solver_config::SolverBackend::Auto) => SolverBackend::Auto,
            Ok(proto::solver_config::SolverBackend::CoinCbc) => SolverBackend::CoinCbc,
            Ok(proto::solver_config::SolverBackend::Highs) => SolverBackend::Highs,
            Err(_) => SolverBackend::Auto,
        };

        SolverConfig {
            backend,
            time_limit: if cfg.time_limit > 0.0 {
                Some(cfg.time_limit)
            } else {
                None
            },
            gap_tolerance: cfg.mip_options.as_ref().and_then(|m| {
                if m.gap_tolerance > 0.0 {
                    Some(m.gap_tolerance)
                } else {
                    None
                }
            }),
            verbose: cfg.verbose,
        }
    } else {
        SolverConfig::default()
    };

    Ok(OptimizationProblem {
        name: proto_prob.problem_name,
        description: proto_prob.description,
        objective,
        constraints,
        variables,
        solver_config,
    })
}

/// Convert domain Solution to protobuf OptimizationResult
pub fn domain_to_proto_solution(
    solution: Solution,
    solver_name: &str,
) -> proto::OptimizationResult {
    let status = match solution.status {
        SolutionStatus::Optimal => proto::SolutionStatus::Optimal as i32,
        SolutionStatus::Feasible => proto::SolutionStatus::Feasible as i32,
        SolutionStatus::Infeasible => proto::SolutionStatus::Infeasible as i32,
        SolutionStatus::Unbounded => proto::SolutionStatus::Unbounded as i32,
        SolutionStatus::TimeLimit => proto::SolutionStatus::TimeLimit as i32,
        SolutionStatus::IterationLimit => proto::SolutionStatus::IterationLimit as i32,
        SolutionStatus::NodeLimit => proto::SolutionStatus::NodeLimit as i32,
        SolutionStatus::Error => proto::SolutionStatus::Error as i32,
        SolutionStatus::Interrupted => proto::SolutionStatus::Interrupted as i32,
    };

    proto::OptimizationResult {
        status,
        optimal_value: solution.optimal_value,
        best_bound: solution.best_bound,
        gap: solution.gap,
        solution_values: solution.variable_values,
        dual_values: solution.dual_values,
        reduced_costs: vec![],
        slack_values: vec![],
        message: solution.message,
        statistics: Some(proto::SolverStatistics {
            simplex_iterations: solution.statistics.simplex_iterations,
            nodes_explored: solution.statistics.nodes_explored,
            solve_time_ms: solution.statistics.solve_time_ms,
            num_variables: solution.statistics.num_variables,
            num_constraints: solution.statistics.num_constraints,
            num_integer_vars: solution.statistics.num_integer_vars,
            num_binary_vars: solution.statistics.num_binary_vars,
            solver_backend: solver_name.to_string(),
        }),
        quality: Some(proto::SolutionQuality {
            max_constraint_violation: solution.quality.max_constraint_violation,
            max_relative_violation: 0.0,
            max_integrality_violation: solution.quality.max_integrality_violation,
            reliability: solution.quality.reliability,
        }),
    }
}
