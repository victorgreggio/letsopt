// Example: Facility Location Problem using gRPC Streaming
//
// Demonstrates streaming for problems with many variables and constraints.
// A logistics company decides which warehouses to open and how to allocate shipments.
//
// Problem: 10 warehouses, 30 customers
// Variables: 10 binary (open/close) + 300 continuous (shipment quantities) = 310 total
// Constraints: 30 (demand) + 10 (capacity) = 40 total
//
// Benefits of streaming:
// - Handles very large problems (exceeding message size limits)
// - Reduces client memory usage by sending incrementally
// - Enables dynamic problem generation

use futures::stream;
use std::io::{self, Write};
use tonic::Request;

pub mod lp_solver {
    tonic::include_proto!("lp_solver");
}

use lp_solver::{
    constraint::ConstraintType, linear_programming_solver_client::LinearProgrammingSolverClient,
    mip_options::MipEmphasis, objective_function::OptimizationType, problem_chunk,
    solver_config::SolverBackend, variable::VariableType, Constraint, Empty, MipOptions,
    ObjectiveFunction, ProblemChunk, ProblemMetadata, SolutionStatus, SolverConfig, Variable,
};

const NUM_WAREHOUSES: usize = 10;
const NUM_CUSTOMERS: usize = 30;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = LinearProgrammingSolverClient::connect("http://127.0.0.1:50051").await?;

    println!("=== Facility Location Problem (gRPC Streaming) ===\n");
    println!("Problem:");
    println!("  • {} potential warehouses", NUM_WAREHOUSES);
    println!("  • {} customers", NUM_CUSTOMERS);
    println!(
        "  • {} variables ({} binary + {} continuous)",
        NUM_WAREHOUSES * (1 + NUM_CUSTOMERS),
        NUM_WAREHOUSES,
        NUM_WAREHOUSES * NUM_CUSTOMERS
    );
    println!("  • {} constraints\n", NUM_CUSTOMERS + NUM_WAREHOUSES);

    // Fetch and display available solvers
    let solvers_response = client.get_available_solvers(Request::new(Empty {})).await?;
    let available_solvers = solvers_response.into_inner().solvers;

    println!("Available Solvers:");
    for (i, solver) in available_solvers.iter().enumerate() {
        if solver.supports_mip {
            println!("  {}. {} (v{})", i + 1, solver.name, solver.version);
        }
    }

    // Prompt for solver selection
    print!(
        "\nSelect solver (1-{}, or 0 for AUTO): ",
        available_solvers.len()
    );
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

    let warehouse_data = generate_warehouse_data();
    let demands = generate_demands();
    let costs = generate_costs();

    println!("Sample data:");
    println!("  Warehouse 1: Fixed=$10,000, Capacity=120");
    println!("  Customer 1: Demand=15 units");
    println!("  Shipping W1→C1: $7/unit\n");

    println!("Streaming problem in chunks...");
    let chunks = create_chunks(
        warehouse_data.clone(),
        demands.clone(),
        costs.clone(),
        solver_backend,
    );
    println!("Sending {} chunks...\n", chunks.len());

    let response = client
        .solve_problem_stream(Request::new(stream::iter(chunks)))
        .await?;
    let result = response.into_inner();

    println!("=== Solution ===\n");

    match SolutionStatus::try_from(result.status) {
        Ok(SolutionStatus::Optimal) | Ok(SolutionStatus::Feasible) => {
            println!("✓ Solution found!\n");

            let mut total_fixed = 0.0;
            let mut open_wh = Vec::new();

            println!("Warehouses to open:");
            for i in 0..NUM_WAREHOUSES {
                if result.solution_values[i] > 0.5 {
                    total_fixed += warehouse_data[i].0;
                    open_wh.push(i);
                    println!(
                        "  ✓ Warehouse {} - Fixed: ${:.0}, Cap: {:.0}",
                        i + 1,
                        warehouse_data[i].0,
                        warehouse_data[i].1
                    );
                }
            }

            let mut total_ship = 0.0;
            let mut shipments = Vec::new();

            for i in 0..NUM_WAREHOUSES {
                if result.solution_values[i] > 0.5 {
                    for j in 0..NUM_CUSTOMERS {
                        let idx = NUM_WAREHOUSES + i * NUM_CUSTOMERS + j;
                        let qty = result.solution_values[idx];
                        if qty > 0.01 {
                            let cost = qty * costs[i][j];
                            total_ship += cost;
                            shipments.push((i, j, qty, cost));
                        }
                    }
                }
            }

            shipments.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());

            println!("\nTop 8 shipments:");
            for (idx, (w, c, q, cost)) in shipments.iter().take(8).enumerate() {
                println!(
                    "  {}. W{} → C{}: {:.1} units @ ${:.0}/u = ${:.0}",
                    idx + 1,
                    w + 1,
                    c + 1,
                    q,
                    costs[*w][*c],
                    cost
                );
            }
            if shipments.len() > 8 {
                println!("  ... and {} more", shipments.len() - 8);
            }

            println!("\n═══════════════════════════");
            println!("  Fixed costs:    ${:>10.0}", total_fixed);
            println!("  Shipping costs: ${:>10.0}", total_ship);
            println!("  ─────────────────────────");
            println!("  Total cost:     ${:>10.0}", result.optimal_value.unwrap());
            println!("═══════════════════════════");

            if let Some(stats) = result.statistics {
                println!("\nPerformance:");
                println!(
                    "  Variables:   {} ({} binary)",
                    stats.num_variables, stats.num_binary_vars
                );
                println!("  Constraints: {}", stats.num_constraints);
                println!("  B&B Nodes:   {}", stats.nodes_explored);
                println!("  Time:        {:.0} ms", stats.solve_time_ms);
            }
        }
        _ => println!("✗ No optimal solution found"),
    }

    Ok(())
}

fn generate_warehouse_data() -> Vec<(f64, f64)> {
    (0..NUM_WAREHOUSES)
        .map(|i| (10000.0 + i as f64 * 1000.0, 120.0 + (i % 4) as f64 * 30.0))
        .collect()
}

fn generate_demands() -> Vec<f64> {
    (0..NUM_CUSTOMERS)
        .map(|i| 15.0 + (i % 8) as f64 * 5.0)
        .collect()
}

fn generate_costs() -> Vec<Vec<f64>> {
    (0..NUM_WAREHOUSES)
        .map(|i| {
            (0..NUM_CUSTOMERS)
                .map(|j| 5.0 + ((i as i32 - j as i32 / 3).abs() + 1) as f64 * 2.0)
                .collect()
        })
        .collect()
}

fn create_chunks(
    wh: Vec<(f64, f64)>,
    dem: Vec<f64>,
    cost: Vec<Vec<f64>>,
    solver_backend: SolverBackend,
) -> Vec<ProblemChunk> {
    let mut chunks = Vec::new();

    // Metadata
    chunks.push(ProblemChunk {
        chunk: Some(problem_chunk::Chunk::Metadata(ProblemMetadata {
            problem_name: "Facility Location".to_string(),
            description: format!("{} wh, {} cust", NUM_WAREHOUSES, NUM_CUSTOMERS),
        })),
    });

    // Config
    chunks.push(ProblemChunk {
        chunk: Some(problem_chunk::Chunk::SolverConfig(SolverConfig {
            solver: solver_backend as i32,
            time_limit: 120.0,
            tolerance: 0.0001,
            max_iterations: 0,
            num_threads: 0,
            verbose: false,
            mip_options: Some(MipOptions {
                gap_tolerance: 0.02,
                max_nodes: 50000,
                max_solutions: 0,
                emphasis: MipEmphasis::Balanced as i32,
                branching: 0,
            }),
            presolve: 0,
        })),
    });

    // Variables: binary for warehouses
    for i in 0..NUM_WAREHOUSES {
        chunks.push(ProblemChunk {
            chunk: Some(problem_chunk::Chunk::Variable(Variable {
                r#type: VariableType::Binary as i32,
                lower_bound: 0.0,
                upper_bound: Some(1.0),
                name: format!("y{}", i),
            })),
        });
    }

    // Variables: continuous for flows
    for i in 0..NUM_WAREHOUSES {
        for j in 0..NUM_CUSTOMERS {
            chunks.push(ProblemChunk {
                chunk: Some(problem_chunk::Chunk::Variable(Variable {
                    r#type: VariableType::Continuous as i32,
                    lower_bound: 0.0,
                    upper_bound: None,
                    name: format!("x{}_{}", i, j),
                })),
            });
        }
    }

    // Objective
    let mut coeffs = Vec::new();
    let mut names = Vec::new();

    for i in 0..NUM_WAREHOUSES {
        coeffs.push(wh[i].0);
        names.push(format!("y{}", i));
    }

    for i in 0..NUM_WAREHOUSES {
        for j in 0..NUM_CUSTOMERS {
            coeffs.push(cost[i][j]);
            names.push(format!("x{}_{}", i, j));
        }
    }

    chunks.push(ProblemChunk {
        chunk: Some(problem_chunk::Chunk::Objective(ObjectiveFunction {
            r#type: OptimizationType::Minimize as i32,
            coefficients: coeffs,
            variable_names: names,
        })),
    });

    // Demand constraints: sum_i(x_ij) >= demand_j
    for j in 0..NUM_CUSTOMERS {
        let mut c = vec![0.0; NUM_WAREHOUSES]; // Binary vars: no contribution

        // Flow vars: x_ij for all i, j
        for _i in 0..NUM_WAREHOUSES {
            for jj in 0..NUM_CUSTOMERS {
                if jj == j {
                    c.push(1.0); // This customer's incoming flow
                } else {
                    c.push(0.0); // Other customers
                }
            }
        }

        chunks.push(ProblemChunk {
            chunk: Some(problem_chunk::Chunk::Constraint(Constraint {
                r#type: ConstraintType::GreaterThanOrEqual as i32,
                coefficients: c,
                bound: dem[j],
                name: format!("dem{}", j),
            })),
        });
    }

    // Capacity constraints: sum_j(x_ij) - cap_i * y_i <= 0
    for i in 0..NUM_WAREHOUSES {
        let mut c = vec![0.0; NUM_WAREHOUSES];
        c[i] = -wh[i].1; // -capacity * y_i

        // Flow vars: x_ij for all warehouses and customers
        for wi in 0..NUM_WAREHOUSES {
            for _j in 0..NUM_CUSTOMERS {
                if wi == i {
                    c.push(1.0); // This warehouse's outflow
                } else {
                    c.push(0.0); // Other warehouses
                }
            }
        }

        chunks.push(ProblemChunk {
            chunk: Some(problem_chunk::Chunk::Constraint(Constraint {
                r#type: ConstraintType::LessThanOrEqual as i32,
                coefficients: c,
                bound: 0.0,
                name: format!("cap{}", i),
            })),
        });
    }

    chunks
}
