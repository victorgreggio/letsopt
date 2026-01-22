#[cfg(feature = "server")]
use tonic::{Request, Response, Status};

#[cfg(feature = "server")]
use super::mappers::{self, lp_solver};

#[cfg(feature = "server")]
use crate::solver::SolverFactory;

/// gRPC service implementation
pub struct GrpcLpSolverService;

impl GrpcLpSolverService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GrpcLpSolverService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "server")]
#[tonic::async_trait]
impl lp_solver::linear_programming_solver_server::LinearProgrammingSolver for GrpcLpSolverService {
    async fn solve_problem(
        &self,
        request: Request<lp_solver::OptimizationProblem>,
    ) -> Result<Response<lp_solver::OptimizationResult>, Status> {
        let proto_problem = request.into_inner();

        println!("📊 Solving problem: {}", proto_problem.problem_name);
        if !proto_problem.description.is_empty() {
            println!("   Description: {}", proto_problem.description);
        }

        // Convert protobuf to domain model
        let domain_problem = mappers::proto_to_domain_problem(proto_problem).map_err(|e| *e)?;

        // Create solver based on problem configuration
        let solver = SolverFactory::create_solver(&domain_problem);
        println!("   Using solver: {}", solver.name());

        // Solve using domain service
        let solution = solver
            .solve(&domain_problem)
            .map_err(|e| Status::internal(format!("Solver error: {}", e)))?;

        println!("✓ Status: {}", solution.status);

        // Convert domain solution to protobuf
        let proto_result = mappers::domain_to_proto_solution(solution, solver.name());

        Ok(Response::new(proto_result))
    }

    async fn solve_problem_stream(
        &self,
        request: Request<tonic::Streaming<lp_solver::ProblemChunk>>,
    ) -> Result<Response<lp_solver::OptimizationResult>, Status> {
        let mut stream = request.into_inner();

        let mut objective: Option<lp_solver::ObjectiveFunction> = None;
        let mut constraints = Vec::new();
        let mut variables = Vec::new();
        let mut solver_config: Option<lp_solver::SolverConfig> = None;
        let mut problem_name = String::new();
        let mut description = String::new();

        // Collect all chunks
        while let Some(chunk) = stream.message().await? {
            match chunk.chunk {
                Some(lp_solver::problem_chunk::Chunk::Objective(obj)) => {
                    objective = Some(obj);
                }
                Some(lp_solver::problem_chunk::Chunk::Constraint(c)) => {
                    constraints.push(c);
                }
                Some(lp_solver::problem_chunk::Chunk::Variable(v)) => {
                    variables.push(v);
                }
                Some(lp_solver::problem_chunk::Chunk::Metadata(m)) => {
                    problem_name = m.problem_name;
                    description = m.description;
                }
                Some(lp_solver::problem_chunk::Chunk::SolverConfig(sc)) => {
                    solver_config = Some(sc);
                }
                None => {}
            }
        }

        // Build complete problem
        let proto_problem = lp_solver::OptimizationProblem {
            objective,
            constraints,
            variables,
            solver_config,
            problem_name,
            description,
        };

        // Reuse solve_problem logic
        let domain_problem = mappers::proto_to_domain_problem(proto_problem).map_err(|e| *e)?;
        let solver = SolverFactory::create_solver(&domain_problem);
        let solution = solver
            .solve(&domain_problem)
            .map_err(|e| Status::internal(format!("Solver error: {}", e)))?;

        let proto_result = mappers::domain_to_proto_solution(solution, solver.name());
        Ok(Response::new(proto_result))
    }

    async fn get_available_solvers(
        &self,
        _request: Request<lp_solver::Empty>,
    ) -> Result<Response<lp_solver::AvailableSolvers>, Status> {
        let solvers = vec![
            lp_solver::SolverInfo {
                name: "COIN-OR CBC".to_string(),
                version: "2.10+".to_string(),
                supports_mip: true,
                supports_lp: true,
                capabilities: vec![
                    "Mixed-Integer Programming".to_string(),
                    "Branch and Bound".to_string(),
                    "Cutting Planes".to_string(),
                    "Primal/Dual Simplex".to_string(),
                ],
            },
            lp_solver::SolverInfo {
                name: "HiGHS".to_string(),
                version: "1.7+".to_string(),
                supports_mip: true,
                supports_lp: true,
                capabilities: vec![
                    "Mixed-Integer Programming".to_string(),
                    "Linear Programming".to_string(),
                    "Primal/Dual Simplex".to_string(),
                    "Interior Point Method".to_string(),
                    "Presolve".to_string(),
                ],
            },
        ];

        Ok(Response::new(lp_solver::AvailableSolvers { solvers }))
    }

    async fn validate_problem(
        &self,
        request: Request<lp_solver::OptimizationProblem>,
    ) -> Result<Response<lp_solver::ValidationResult>, Status> {
        let proto_problem = request.into_inner();
        let domain_problem = mappers::proto_to_domain_problem(proto_problem).map_err(|e| *e)?;

        // Use the default solver for validation
        let solver = SolverFactory::default_solver();

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Use domain service validation
        match solver.validate(&domain_problem) {
            Ok(_) => {
                // Additional warnings
                if domain_problem.constraints.is_empty() {
                    warnings.push("Problem has no constraints (may be unbounded)".to_string());
                }

                let num_integer = domain_problem.num_integer_variables();
                if num_integer > 100 {
                    warnings.push(format!(
                        "Problem has {} integer variables, may be slow to solve",
                        num_integer
                    ));
                }
            }
            Err(e) => {
                errors.push(e.to_string());
            }
        }

        let estimated_difficulty = if domain_problem.is_mixed_integer() {
            (domain_problem.num_integer_variables() as f64 / 1000.0).min(1.0)
        } else {
            (domain_problem.num_variables() as f64 / 10000.0).min(0.5)
        };

        Ok(Response::new(lp_solver::ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            num_variables: domain_problem.num_variables() as u32,
            num_constraints: domain_problem.constraints.len() as u32,
            num_integer_vars: domain_problem.num_integer_variables() as u32,
            estimated_difficulty,
        }))
    }
}
