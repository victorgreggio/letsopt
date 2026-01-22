# Quick Start Guide

## Installation

### Clone the Repository

```bash
git clone https://github.com/victorgreggio/letsopt.git
cd letsopt
```

## Running the Server

### 🐳 Using Docker (Recommended)

**1. Start the server:**
```bash
docker-compose up -d
```

**2. Check if it's running:**
```bash
docker-compose ps
```

**3. View logs:**
```bash
docker-compose logs -f letsopt-server
```

**4. Stop the server:**
```bash
docker-compose down
```

**Manual Docker commands:**
```bash
# Build
docker build -t letsopt:latest .

# Run
docker run -d -p 50051:50051 --name letsopt-server letsopt:latest

# Check logs
docker logs -f letsopt-server

# Stop
docker stop letsopt-server
docker rm letsopt-server
```

### 🔧 Building from Source

**1. Build the project:**

```bash
cargo build --release
```

**2. Start the Server:**

```bash
cargo run --release --bin letsopt-server
```

You should see:

```
Linear & Mixed-Integer Programming Solver gRPC Server
=======================================================
Powered by: COIN-OR CBC
Listening on: 0.0.0.0:50051

Supported Features:
  ✓ Linear Programming (LP)
  ✓ Mixed-Integer Programming (MIP)
  ✓ Binary Variables
  ✓ Integer Variables
  ✓ Branch-and-Cut Algorithm

Ready to solve optimization problems!
```

The server is now listening on port 50051.

## Testing the Server

### Test with Examples

**Note:** If using Docker, you can still run examples from your host machine with `cargo run --example client` since port 50051 is exposed.

**In a new terminal**, run the example clients:

### Linear Programming Example (Production Planning)

```bash
cargo run --example client
```

Expected output:

```
=== Production Planning Problem ===

Solving problem to solver...

=== Solution ===

✓ Optimal solution found!

Optimal Production Plan:
  Chairs:  0.00 units
  Tables:  33.33 units

Maximum Profit: $1666.67

Solver Statistics:
  Variables:   2
  Constraints: 2
  Solve Time:  0.17 ms
```

#### Mixed-Integer Programming Example (Knapsack)

```bash
cargo run --example mip_client
```

Expected output:

```
=== Knapsack Problem (Mixed-Integer Programming) ===

A hiker has 15 kg capacity. Which items to pack?

Available Items:
┌────────┬────────────┬───────────┐
│ Item   │ Weight(kg) │ Value ($) │
├────────┼────────────┼───────────┤
│ Tent   │      7.0   │    150.0  │
│ Stove  │      3.0   │     90.0  │
│ Food   │      4.0   │    120.0  │
│ Water  │      5.0   │    100.0  │
│ Camera │      2.0   │     80.0  │
└────────┴────────────┴───────────┘

Knapsack Capacity: 15 kg

Solving with COIN-OR CBC (MIP solver)...

=== Solution ===

✓ Optimal solution found!

Items to Pack:
  ✓ Tent   - Weight: 7.0 kg, Value: $150
  ✓ Stove  - Weight: 3.0 kg, Value: $90
  ✓ Camera - Weight: 2.0 kg, Value: $80
  ✗ Food   - (not selected)
  ✗ Water  - (not selected)

Summary:
  Total Weight:  12.0 / 15.0 kg
  Total Value:   $320
  Maximum Value: $320
```

## Performance Tuning

### For Large LP Problems

```rust
SolverConfig {
    presolve: PresolveLevel::On,  // Enable presolve
    num_threads: 0,  // Use all available cores
    ..Default::default()
}
```

### For MIP Problems

```rust
SolverConfig {
    mip_options: Some(MipOptions {
        gap_tolerance: 0.01,  // 1% gap acceptable
        emphasis: MipEmphasis::Balanced,
        branching: BranchingStrategy::PseudoCost,
        max_nodes: 100000,
        ..Default::default()
    }),
    time_limit: 300.0,  // 5 minutes max
    ..Default::default()
}
```