// Example: Mixed-Integer Programming (MIP) - 0/1 Knapsack Problem
//
// A hiker has a knapsack with capacity of 15 kg.
// There are 5 items to choose from:
//
// Item   | Weight (kg) | Value ($)
// -------|-------------|----------
// Tent   |     7       |   150
// Stove  |     3       |    90
// Food   |     4       |   120
// Water  |     5       |   100
// Camera |     2       |    80
//
// Question: Which items should the hiker take to maximize value?
//
// Decision Variables: x_i ∈ {0, 1} for each item (binary: take it or not)
// Maximize: 150*x₁ + 90*x₂ + 120*x₃ + 100*x₄ + 80*x₅
// Subject to: 7*x₁ + 3*x₂ + 4*x₃ + 5*x₄ + 2*x₅ ≤ 15 (weight limit)

use std::io::{self, Write};
use tonic::Request;

pub mod lp_solver {
    tonic::include_proto!("lp_solver");
}

use lp_solver::{
    constraint::ConstraintType, linear_programming_solver_client::LinearProgrammingSolverClient,
    objective_function::OptimizationType, solver_config::SolverBackend, variable::VariableType,
    Constraint, Empty, ObjectiveFunction, OptimizationProblem, SolutionStatus, SolverConfig,
    Variable,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = LinearProgrammingSolverClient::connect("http://127.0.0.1:50051").await?;

    println!("=== Knapsack Problem (Mixed-Integer Programming) ===\n");
    println!("A hiker has 15 kg capacity. Which items to pack?\n");

    // Fetch available solvers
    let solvers_response = client.get_available_solvers(Request::new(Empty {})).await?;
    let available_solvers = solvers_response.into_inner().solvers;

    // Display available solvers with MIP support
    println!("Available Solvers for MIP:");
    let mut valid_choices = vec![];
    for (i, solver) in available_solvers.iter().enumerate() {
        if solver.supports_mip {
            valid_choices.push(i + 1);
            println!("  {}. {} (v{})", i + 1, solver.name, solver.version);
        }
    }

    // Prompt user to select solver
    print!("\nSelect solver (1-{}, or 0 for AUTO): ", available_solvers.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice: usize = input.trim().parse().unwrap_or(0);

    let solver_backend = match choice {
        0 => {
            println!("Using AUTO selection (HiGHS)\n");
            SolverBackend::Auto
        }
        1 => {
            println!("Using COIN-OR CBC\n");
            SolverBackend::CoinCbc
        }
        2 => {
            println!("Using HiGHS\n");
            SolverBackend::Highs
        }
        _ => {
            println!("Invalid choice, using AUTO\n");
            SolverBackend::Auto
        }
    };

    let items = vec![
        ("Tent", 7.0, 150.0),
        ("Stove", 3.0, 90.0),
        ("Food", 4.0, 120.0),
        ("Water", 5.0, 100.0),
        ("Camera", 2.0, 80.0),
    ];

    // Print items
    println!("Available Items:");
    println!("┌────────┬────────────┬───────────┐");
    println!("│ Item   │ Weight(kg) │ Value ($) │");
    println!("├────────┼────────────┼───────────┤");
    for (name, weight, value) in &items {
        println!("│ {:6} │    {:5.1}   │   {:6.0}  │", name, weight, value);
    }
    println!("└────────┴────────────┴───────────┘");
    println!("\nKnapsack Capacity: 15 kg\n");

    // Define binary variables for each item
    let mut variables = Vec::new();
    let mut weights = Vec::new();
    let mut values = Vec::new();

    for (name, weight, value) in &items {
        variables.push(Variable {
            r#type: VariableType::Binary as i32, // Binary: 0 or 1
            lower_bound: 0.0,
            upper_bound: Some(1.0),
            name: name.to_string(),
        });
        weights.push(*weight);
        values.push(*value);
    }

    // Objective: Maximize total value
    let objective = ObjectiveFunction {
        r#type: OptimizationType::Maximize as i32,
        coefficients: values.clone(),
        variable_names: items.iter().map(|(name, _, _)| name.to_string()).collect(),
    };

    // Constraint: Total weight ≤ 15 kg
    let constraints = vec![Constraint {
        r#type: ConstraintType::LessThanOrEqual as i32,
        coefficients: weights,
        bound: 15.0,
        name: "Weight capacity".to_string(),
    }];

    // Use selected solver
    let solver_config = SolverConfig {
        solver: solver_backend as i32,
        time_limit: 60.0,
        tolerance: 0.0001,
        max_iterations: 0,
        num_threads: 0,
        verbose: false,
        mip_options: None,
        presolve: 0, // Auto
    };

    // Build the problem
    let problem = OptimizationProblem {
        objective: Some(objective),
        constraints,
        variables,
        solver_config: Some(solver_config),
        problem_name: "Knapsack Problem".to_string(),
        description: "0/1 Knapsack with 5 items and 15 kg capacity".to_string(),
    };

    // Solve
    println!("Solving with selected solver...\n");
    let response = client.solve_problem(Request::new(problem)).await?;
    let result = response.into_inner();

    // Display results
    println!("=== Solution ===\n");

    match SolutionStatus::try_from(result.status) {
        Ok(SolutionStatus::Optimal) | Ok(SolutionStatus::Feasible) => {
            println!("✓ Optimal solution found!\n");
            println!("Items to Pack:");

            let mut total_weight = 0.0;
            let mut total_value = 0.0;

            for (i, (name, weight, value)) in items.iter().enumerate() {
                if result.solution_values[i] > 0.5 {
                    // Binary: 1 means take it
                    println!(
                        "  ✓ {:6} - Weight: {:.1} kg, Value: ${:.0}",
                        name, weight, value
                    );
                    total_weight += weight;
                    total_value += value;
                } else {
                    println!("  ✗ {:6} - (not selected)", name);
                }
            }

            println!("\nSummary:");
            println!("  Total Weight:  {:.1} / 15.0 kg", total_weight);
            println!("  Total Value:   ${:.0}", total_value);
            println!("  Maximum Value: ${:.0}", result.optimal_value.unwrap());

            if let Some(stats) = result.statistics {
                println!("\nSolver Statistics:");
                println!("  Solver:      {}", stats.solver_backend);
                println!(
                    "  Variables:   {} ({} binary)",
                    stats.num_variables, stats.num_binary_vars
                );
                println!("  Constraints: {}", stats.num_constraints);
                println!("  Nodes:       {}", stats.nodes_explored);
                println!("  Solve Time:  {:.2} ms", stats.solve_time_ms);
            }

            if let Some(quality) = result.quality {
                println!("\nSolution Quality:");
                println!("  Reliability: {:.2}%", quality.reliability * 100.0);
            }
        }
        Ok(SolutionStatus::Infeasible) => {
            println!("✗ Problem is infeasible");
            println!("  No combination of items satisfies all constraints");
        }
        Ok(SolutionStatus::Unbounded) => {
            println!("⚠ Problem is unbounded");
            println!("  This shouldn't happen for a knapsack problem!");
        }
        _ => {
            println!("✗ Solver error or timeout");
        }
    }

    println!("\nMessage: {}", result.message);

    Ok(())
}
