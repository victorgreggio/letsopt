# LetsOpt - Linear & Mixed-Integer Programming gRPC Service

A gRPC-based optimization service built on **COIN-OR CBC**. Solve complex Linear Programming (LP) and Mixed-Integer Programming (MIP) problems from any gRPC enabled programming language.

## Overview

LetsOpt provides a production-ready gRPC API for solving optimization problems. The API uses mathematical terminology instead of programming-specific concepts, making it accessible to engineers, analysts, and domain experts.

### What is Linear/Mixed-Integer Programming?

Optimization problems of the form:

```
Maximize (or Minimize): c₁*x₁ + c₂*x₂ + ... + cₙ*xₙ

Subject to:
  a₁₁*x₁ + a₁₂*x₂ + ... + a₁ₙ*xₙ  {≤, =, ≥}  b₁
  a₂₁*x₁ + a₂₂*x₂ + ... + a₂ₙ*xₙ  {≤, =, ≥}  b₂
  ...
  
Where variables can be:
  xᵢ ∈ ℝ        (continuous - any real number)
  xᵢ ∈ ℤ        (integer - whole numbers)
  xᵢ ∈ {0, 1}   (binary - yes/no decisions)
```

**Components:**
- **Decision Variables** (x₁, x₂, ...): The values you want to find
- **Objective Function** (c₁, c₂, ...): What to maximize or minimize
- **Constraints**: Limitations on the variables
- **Variable Types**: Continuous, integer, or binary

## Features

- 🎯 **Engineer-Friendly API**: Mathematical terminology (objective, constraints, variables)
- 🚀 **Industrial Strength**: Powered by COIN-OR CBC (Branch-and-Cut algorithm)
- 🔢 **Full MIP Support**: Integer and binary variables
- 📡 **gRPC Interface**: Platform-independent, any language
- 📊 **Rich Results**: Optimal values, bounds, gaps, dual values, quality metrics
- ⚡ **Advanced Options**: Time limits, gap tolerance, branching strategies
- 🔍 **Problem Validation**: Validate before solving
- 📈 **Multiple Solvers**: Auto-select or choose specific backend(only one implemented for now)

## Quick Start

### Option 1: Using Docker (Recommended)

**Start the server with Docker Compose:**
```bash
docker-compose up -d
```

**Or build and run manually:**
```bash
# Build the image
docker build -t letsopt:latest .

# Run the container
docker run -d -p 50051:50051 --name letsopt-server letsopt:latest
```

**Check logs:**
```bash
docker logs -f letsopt-server
```

**Stop the server:**
```bash
docker-compose down
# or
docker stop letsopt-server
```

### Option 2: Build from Source

**Prerequisites:**
- Rust 1.75+ (`rustup` recommended)
- Protocol Buffers compiler (`protoc`)
- C++ compiler (for COIN-OR CBC)

**Start the server:**
```bash
cargo run --bin letsopt-server
```

Server starts on `0.0.0.0:50051`

### Run Examples

**Linear Programming (Production Planning):**
```bash
cargo run --example client
```

**Mixed-Integer Programming (Knapsack Problem):**
```bash
cargo run --example mip_client
```

**Streaming API (Facility Location):**
```bash
cargo run --example stream_client
```

The streaming example demonstrates sending large problems in chunks:
- Handles problems with thousands of variables/constraints
- Useful when data exceeds message size limits
- Enables dynamic problem generation
- Reduces client memory usage

See [QUICKSTART.md](QUICKSTART.md) for detailed setup instructions.

## Example: 0/1 Knapsack Problem (MIP)

A hiker has a 15 kg knapsack. Which items should they pack to maximize value?

| Item   | Weight (kg) | Value ($) |
|--------|-------------|-----------|
| Tent   | 7           | 150       |
| Stove  | 3           | 90        |
| Food   | 4           | 120       |
| Water  | 5           | 100       |
| Camera | 2           | 80        |

**Mathematical Formulation:**

```
Decision Variables: xᵢ ∈ {0, 1} for each item (take it or not)

Maximize: 150*x₁ + 90*x₂ + 120*x₃ + 100*x₄ + 80*x₅

Subject to: 7*x₁ + 3*x₂ + 4*x₃ + 5*x₄ + 2*x₅ ≤ 15  (weight limit)
```

**Code Example:**

```rust
use lp_solver::{
    Variable, VariableType, Constraint, ConstraintType,
    ObjectiveFunction, OptimizationType, OptimizationProblem,
};

// Define binary variables (0 or 1 for each item)
let variables = vec![
    Variable {
        r#type: VariableType::Binary as i32,
        lower_bound: 0.0,
        upper_bound: Some(1.0),
        name: "Tent".to_string(),
    },
    // ... (4 more items)
];

// Objective: Maximize total value
let objective = ObjectiveFunction {
    r#type: OptimizationType::Maximize as i32,
    coefficients: vec![150.0, 90.0, 120.0, 100.0, 80.0],
    variable_names: vec!["Tent", "Stove", "Food", "Water", "Camera"],
};

// Constraint: Total weight ≤ 15 kg
let constraints = vec![
    Constraint {
        r#type: ConstraintType::LessThanOrEqual as i32,
        coefficients: vec![7.0, 3.0, 4.0, 5.0, 2.0],
        bound: 15.0,
        name: "Weight capacity".to_string(),
    },
];

let problem = OptimizationProblem {
    objective: Some(objective),
    constraints,
    variables,
    solver_config: None, // Use defaults
    problem_name: "Knapsack Problem".to_string(),
    description: "Maximize value within weight limit".to_string(),
};

let result = client.solve_problem(problem).await?;
```

**Solution:**

```
✓ Optimal solution found!

Items to Pack:
  ✓ Tent   - Weight: 7.0 kg, Value: $150
  ✓ Stove  - Weight: 3.0 kg, Value: $90
  ✓ Camera - Weight: 2.0 kg, Value: $80
  ✗ Food   - (not selected)
  ✗ Water  - (not selected)

Total Weight:  12.0 / 15.0 kg
Total Value:   $320
Maximum Value: $320

Solver Statistics:
  Variables:   5 (5 binary)
  Constraints: 1
  Nodes:       13
  Solve Time:  2.34 ms
```

## API Reference

### Variable Types

```protobuf
message Variable {
  enum VariableType {
    CONTINUOUS = 0;  // x ∈ ℝ (any real number)
    INTEGER = 1;     // x ∈ ℤ (whole numbers)
    BINARY = 2;      // x ∈ {0, 1} (yes/no decisions)
  }
  
  VariableType type = 1;
  double lower_bound = 2;
  optional double upper_bound = 3;
  string name = 4;
}
```

### Optimization Problem

```protobuf
message OptimizationProblem {
  ObjectiveFunction objective = 1;      // What to optimize
  repeated Constraint constraints = 2;   // Limitations
  repeated Variable variables = 3;       // Variable definitions
  SolverConfig solver_config = 4;       // Solver options
  string problem_name = 5;
  string description = 6;
}
```

### Solver Configuration

```protobuf
message SolverConfig {
  enum SolverBackend {
    AUTO = 0;       // Auto-select best solver
    COIN_CBC = 1;   // COIN-OR CBC (MIP solver)
  }
  
  SolverBackend solver = 1;
  double time_limit = 2;          // Seconds (0 = no limit)
  double tolerance = 3;           // Solution tolerance
  uint64 max_iterations = 4;      // Max iterations (0 = no limit)
  uint32 num_threads = 5;         // Threads (0 = auto)
  bool verbose = 6;               // Solver output
  MipOptions mip_options = 7;     // MIP-specific options
  PresolveLevel presolve = 8;     // Presolve level
}
```

### MIP Options

```protobuf
message MipOptions {
  double gap_tolerance = 1;       // Stop when within X% of optimal
  uint64 max_nodes = 2;           // Max branch-and-bound nodes
  uint32 max_solutions = 3;       // Max solutions to find
  
  enum MipEmphasis {
    BALANCED = 0;              // Balance speed and optimality
    FEASIBILITY = 1;           // Find solutions quickly
    OPTIMALITY = 2;            // Prove optimality
    HIDDEN_FEASIBILITY = 3;    // Hard-to-find solutions
  }
  MipEmphasis emphasis = 4;
  
  enum BranchingStrategy {
    AUTO_BRANCHING = 0;
    PSEUDO_COST = 1;
    STRONG_BRANCHING = 2;
  }
  BranchingStrategy branching = 5;
}
```

### Optimization Result

```protobuf
message OptimizationResult {
  SolutionStatus status = 1;          // Solution status
  optional double optimal_value = 2;  // Objective value at optimum
  optional double best_bound = 3;     // Best bound (MIP)
  optional double gap = 4;            // Optimality gap (%)
  repeated double solution_values = 5; // Values for each variable
  repeated double dual_values = 6;     // Shadow prices (LP)
  repeated double reduced_costs = 7;   // Reduced costs (LP)
  repeated double slack_values = 8;    // Constraint slack
  string message = 9;                  // Human-readable message
  SolverStatistics statistics = 10;    // Solver stats
  SolutionQuality quality = 11;        // Solution quality metrics
}

enum SolutionStatus {
  OPTIMAL = 0;           // Found optimal solution
  FEASIBLE = 1;          // Found feasible solution (may not be optimal)
  INFEASIBLE = 2;        // No solution exists
  UNBOUNDED = 3;         // Objective unbounded
  TIME_LIMIT = 4;        // Time limit reached
  ITERATION_LIMIT = 5;   // Iteration limit reached
  NODE_LIMIT = 6;        // Node limit reached (MIP)
  ERROR = 7;             // Solver error
  INTERRUPTED = 8;       // User interrupted
}
```

### RPC Methods

```protobuf
service LinearProgrammingSolver {
  // Solve an optimization problem
  rpc SolveProblem(OptimizationProblem) returns (OptimizationResult);
  
  // Stream large problems in chunks
  rpc SolveProblemStream(stream ProblemChunk) returns (OptimizationResult);
  
  // Get available solver backends
  rpc GetAvailableSolvers(Empty) returns (AvailableSolvers);
  
  // Validate problem without solving
  rpc ValidateProblem(OptimizationProblem) returns (ValidationResult);
}
```

## Use Cases

### Linear Programming (LP) - Continuous Variables

1. **Production Planning**: How many units to produce?
2. **Resource Allocation**: How to distribute limited resources?
3. **Transportation**: Minimize shipping costs
4. **Diet Optimization**: Meet nutrition goals at minimum cost
5. **Portfolio Optimization**: Allocate investments

### Mixed-Integer Programming (MIP) - Integer/Binary Variables

1. **Knapsack Problems**: What items to select?
2. **Facility Location**: Where to build warehouses? (binary)
3. **Job Scheduling**: Assign jobs to time slots (integer)
4. **Vehicle Routing**: Optimize delivery routes (integer + binary)
5. **Production Lot Sizing**: How many production batches? (integer)
6. **Capital Budgeting**: Which projects to fund? (binary)
7. **Network Design**: Which connections to build? (binary)
8. **Workforce Scheduling**: Assign shifts to workers (binary)
9. **Cutting Stock**: Minimize material waste (integer)
10. **Bin Packing**: Pack items into minimum bins (binary)

## Solution Status Guide

| Status | Meaning | Action |
|--------|---------|--------|
| **OPTIMAL** | Found the best solution | ✓ Use solution values |
| **FEASIBLE** | Found a good solution, might not be best | ✓ Use solution, check gap |
| **INFEASIBLE** | No solution satisfies all constraints | ✗ Relax constraints or check model |
| **UNBOUNDED** | Objective can improve infinitely | ✗ Add bounds or constraints |
| **TIME_LIMIT** | Ran out of time | ✓ Use best solution found, or increase limit |
| **NODE_LIMIT** | Explored max nodes | ✓ Use best solution found, or increase limit |

## Architecture

```
┌─────────────────┐
│ Client App      │  (Any Language)
│ Python/JS/Java  │  Define problem in mathematical terms
│ Go/C#/C++/etc.  │  - Variables (continuous/integer/binary)
└────────┬────────┘  - Objective (maximize/minimize)
         │           - Constraints (≤, =, ≥)
         │ gRPC
         ▼
┌─────────────────┐
│ LetsOpt Server  │  (Rust)
│ gRPC Service    │  Translate to solver format
└────────┬────────┘  Handle configuration
         │
         ▼
┌─────────────────┐
│ COIN-OR CBC     │  (C++)
│ Solver Engine   │  Branch-and-Cut algorithm
└─────────────────┘  Cutting planes, heuristics
         │
         ▼
    Solution + Statistics
```

## Performance Characteristics

### COIN-OR CBC

- **Algorithm**: Branch-and-Cut with cutting planes
- **LP Size**: Efficiently handles 10,000+ variables and constraints
- **MIP Size**: Handles 1,000+ integer variables (problem-dependent)
- **Speed**: Modern heuristics find good solutions quickly
- **Optimality**: Proven optimal solutions with gap tolerance

### When to Use

- ✅ Small to medium MIP problems (< 1,000 integer vars)
- ✅ Large LP problems (10,000+ continuous vars)
- ✅ Need proven optimal solutions
- ✅ Open-source requirements
- ✅ Production environments

## Documentation

- **[README.md](README.md)** - This file - Complete API reference and examples
- **[QUICKSTART.md](QUICKSTART.md)** - Step-by-step setup guide

## Project Structure

```
letsopt/
├── Dockerfile             # Multi-stage Docker build
├── docker-compose.yml     # Docker Compose configuration
├── .dockerignore          # Docker build exclusions
├── Cargo.toml             # Rust dependencies
├── build.rs               # Build script (protobuf codegen)
├── QUICKSTART.md          # Setup guide
├── README.md              # This file
├── proto/
│   └── lp_solver.proto    # gRPC API definition
├── src/
│   ├── main.rs            # Server entry point
│   ├── lib.rs             # Library exports
│   ├── domain/            # Business logic (SOLID/DDD)
│   │   ├── models.rs
│   │   ├── value_objects.rs
│   │   └── solver_service.rs
│   ├── application/       # gRPC handlers
│   │   ├── grpc_service.rs
│   │   └── mappers.rs
│   ├── solver/            # Solver adapters
│   │   └── coin_cbc_solver.rs
│   └── infrastructure/    # Server config
│       └── server.rs
├── examples/
│   ├── client.rs          # LP example
│   └── mip_client.rs      # MIP example
└── docs/
    
```