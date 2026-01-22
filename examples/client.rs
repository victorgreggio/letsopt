// Example client demonstrating how to use the LP Solver gRPC service
//
// This example solves a simple production planning problem:
// A factory produces two products: chairs and tables
// - Each chair requires 2 hours of labor and yields $30 profit
// - Each table requires 3 hours of labor and yields $50 profit
// - Maximum 100 hours of labor available
// - Maximum 40 units total (storage constraint)
//
// Objective: Maximize profit
// Variables: x1 = chairs, x2 = tables
// Maximize: 30*x1 + 50*x2
// Subject to:
//   2*x1 + 3*x2 <= 100  (labor hours)
//   x1 + x2 <= 40       (storage)
//   x1, x2 >= 0         (non-negativity)

use tonic::Request;

pub mod lp_solver {
    tonic::include_proto!("lp_solver");
}

use lp_solver::{
    constraint::ConstraintType, linear_programming_solver_client::LinearProgrammingSolverClient,
    objective_function::OptimizationType, variable::VariableType, Constraint, ObjectiveFunction,
    OptimizationProblem, SolutionStatus, Variable,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the gRPC server
    let mut client = LinearProgrammingSolverClient::connect("http://127.0.0.1:50051").await?;

    println!("=== Production Planning Problem ===\n");

    // Define decision variables (continuous, non-negative)
    let variables = vec![
        Variable {
            r#type: VariableType::Continuous as i32,
            lower_bound: 0.0,
            upper_bound: None, // No upper limit
            name: "chairs".to_string(),
        },
        Variable {
            r#type: VariableType::Continuous as i32,
            lower_bound: 0.0,
            upper_bound: None,
            name: "tables".to_string(),
        },
    ];

    // Define the objective: Maximize 30*x1 + 50*x2
    let objective = ObjectiveFunction {
        r#type: OptimizationType::Maximize as i32,
        coefficients: vec![30.0, 50.0],
        variable_names: vec!["chairs".to_string(), "tables".to_string()],
    };

    // Define constraints
    let constraints = vec![
        // Labor constraint: 2*x1 + 3*x2 <= 100
        Constraint {
            r#type: ConstraintType::LessThanOrEqual as i32,
            coefficients: vec![2.0, 3.0],
            bound: 100.0,
            name: "Labor hours limit".to_string(),
        },
        // Storage constraint: x1 + x2 <= 40
        Constraint {
            r#type: ConstraintType::LessThanOrEqual as i32,
            coefficients: vec![1.0, 1.0],
            bound: 40.0,
            name: "Storage capacity".to_string(),
        },
    ];

    // Build the problem
    let problem = OptimizationProblem {
        objective: Some(objective),
        constraints,
        variables,
        solver_config: None, // Use default solver
        problem_name: "Factory Production Planning".to_string(),
        description: "Maximize profit from producing chairs and tables".to_string(),
    };

    // Solve the problem
    println!("Sending problem to solver...\n");
    let response = client.solve_problem(Request::new(problem)).await?;
    let result = response.into_inner();

    // Display results
    println!("=== Solution ===\n");

    match SolutionStatus::try_from(result.status) {
        Ok(SolutionStatus::Optimal) => {
            println!("✓ Optimal solution found!");
            println!("\nOptimal Production Plan:");
            println!("  Chairs:  {:.2} units", result.solution_values[0]);
            println!("  Tables:  {:.2} units", result.solution_values[1]);
            println!("\nMaximum Profit: ${:.2}", result.optimal_value.unwrap());

            if let Some(stats) = result.statistics {
                println!("\nSolver Statistics:");
                println!("  Variables:   {}", stats.num_variables);
                println!("  Constraints: {}", stats.num_constraints);
                println!("  Solve Time:  {:.2} ms", stats.solve_time_ms);
            }
        }
        Ok(SolutionStatus::Infeasible) => {
            println!("✗ Problem is infeasible");
            println!("  No solution satisfies all constraints");
        }
        Ok(SolutionStatus::Unbounded) => {
            println!("⚠ Problem is unbounded");
            println!("  Profit can be increased infinitely");
        }
        _ => {
            println!("✗ Solver error");
        }
    }

    println!("\nMessage: {}", result.message);

    Ok(())
}
