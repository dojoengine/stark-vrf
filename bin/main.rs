use ark_ec::{short_weierstrass::SWCurveConfig, CurveGroup};
use ark_ff::MontFp;
use clap::{Parser, Subcommand};
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;
use serde::{Deserialize, Serialize};
use stark_vrf::{
    curve::{ScalarField, StarkCurve},
    StarkVRF,
};
use std::str::FromStr;
use tonic::{transport::Server, Request, Response, Status};

pub mod stark_vrf {
    tonic::include_proto!("stark_vrf");
}

use stark_vrf::{
    stark_vrf_server::{StarkVrf, StarkVrfServer},
    GenerateProofRequest, GenerateProofResponse, VerifyProofRequest, VerifyProofResponse,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Prove {
        #[arg(short, long)]
        secret_key: String,
        #[arg(short, long)]
        seed: String,
    },
    Verify {
        #[arg(short, long)]
        public_key: String,
        #[arg(short, long)]
        proof: String,
        #[arg(short, long)]
        seed: String,
    },
    Serve {
        #[arg(short, long, default_value = "127.0.0.1:3030")]
        address: String,
    },
    GrpcServe {
        #[arg(short, long, default_value = "127.0.0.1:50051")]
        address: String,
    },
}

#[derive(Serialize, Deserialize)]
struct ProofResponse {
    gamma: String,
    c: String,
    s: String,
    beta: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prove { secret_key, seed } => {
            let (proof, beta) = generate_proof(&secret_key, &seed)?;
            println!("Proof: {}", serde_json::to_string_pretty(&proof)?);
            println!("Beta: {}", beta);
        }
        Commands::Verify {
            public_key,
            proof,
            seed,
        } => {
            let is_valid = verify_proof(&public_key, &proof, &seed)?;
            println!("Proof is valid: {}", is_valid);
        }
        Commands::Serve { address } => {
            serve_rpc(&address)?;
        }
        Commands::GrpcServe { address } => {
            serve_grpc(&address).await?;
        }
    }

    Ok(())
}

// ... (keep the existing generate_proof, verify_proof, and serve_rpc functions)

struct StarkVrfService;

#[tonic::async_trait]
impl StarkVrf for StarkVrfService {
    async fn generate_proof(
        &self,
        request: Request<GenerateProofRequest>,
    ) -> Result<Response<GenerateProofResponse>, Status> {
        let req = request.into_inner();
        match generate_proof(&req.secret_key, &req.seed) {
            Ok((proof, beta)) => Ok(Response::new(GenerateProofResponse {
                proof: serde_json::to_string(&proof).map_err(|e| Status::internal(e.to_string()))?,
                beta,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn verify_proof(
        &self,
        request: Request<VerifyProofRequest>,
    ) -> Result<Response<VerifyProofResponse>, Status> {
        let req = request.into_inner();
        match verify_proof(&req.public_key, &req.proof, &req.seed) {
            Ok(is_valid) => Ok(Response::new(VerifyProofResponse { is_valid })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

async fn serve_grpc(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = address.parse()?;
    let stark_vrf_service = StarkVrfService;

    println!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(StarkVrfServer::new(stark_vrf_service))
        .serve(addr)
        .await?;

    Ok(())
}