use ark_ec::{short_weierstrass::SWCurveConfig, CurveGroup};
use ark_ff::MontFp;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use stark_vrf::{
    curve::{ScalarField, StarkCurve},
    StarkVRF,
};
use std::str::FromStr;
use tonic::{transport::Server, Request, Response, Status};

pub mod stark_vrf_proto {
    tonic::include_proto!("stark_vrf");
}

use stark_vrf_proto::{
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
    // Verify {
    //     #[arg(short, long)]
    //     public_key: String,
    //     #[arg(short, long)]
    //     proof: String,
    //     #[arg(short, long)]
    //     seed: String,
    // },
    Serve {
        #[arg(short, long, default_value = "[::1]:50051")]
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
        // Commands::Verify {
        //     public_key,
        //     proof,
        //     seed,
        // } => {
        //     let is_valid = verify_proof(&public_key, &proof, &seed)?;
        //     println!("Proof is valid: {}", is_valid);
        // }
        Commands::Serve { address } => {
            serve_grpc(&address).await?;
        }
    }

    Ok(())
}

fn generate_proof(
    secret_key: &str,
    seed: &str,
) -> Result<(ProofResponse, String), Box<dyn std::error::Error>> {
    let secret_key = ScalarField::from_str(secret_key)?;
    let public_key = (StarkCurve::GENERATOR * secret_key).into_affine();
    let seed = vec![MontFp!(seed)];

    let ecvrf = StarkVRF::new(public_key)?;
    let proof = ecvrf.prove(&secret_key, &seed)?;
    let beta = ecvrf.proof_to_hash(&proof)?;

    let response = ProofResponse {
        gamma: proof.0.to_string(),
        c: proof.1.to_string(),
        s: proof.2.to_string(),
        beta: beta.to_string(),
    };

    Ok((response, beta.to_string()))
}

// fn verify_proof(
//     public_key: &str,
//     proof: &str,
//     seed: &str,
// ) -> Result<bool, Box<dyn std::error::Error>> {
//     let public_key: StarkCurve = public_key.try_into()?;
//     let proof: ProofResponse = serde_json::from_str(proof)?;
//     let seed = vec![MontFp!(seed)];

//     let ecvrf = StarkVRF::new(public_key)?;
//     let proof = (
//         StarkCurve::from_str(&proof.gamma)?,
//         ScalarField::from_str(&proof.c)?,
//         ScalarField::from_str(&proof.s)?,
//     );

//     Ok(ecvrf.verify(&proof, &seed).is_ok())
// }

#[derive(Debug, Default)]
pub struct StarkVrfService {}

#[tonic::async_trait]
impl StarkVrf for StarkVrfService {
    async fn generate_proof(
        &self,
        request: Request<GenerateProofRequest>,
    ) -> Result<Response<GenerateProofResponse>, Status> {
        let req = request.into_inner();
        match generate_proof(&req.secret_key, &req.seed) {
            Ok((proof, beta)) => {
                let response = GenerateProofResponse {
                    gamma: proof.gamma,
                    c: proof.c,
                    s: proof.s,
                    beta,
                };
                Ok(Response::new(response))
            }
            Err(_) => Err(Status::internal("Failed to generate proof")),
        }
    }

    // async fn verify_proof(
    //     &self,
    //     request: Request<VerifyProofRequest>,
    // ) -> Result<Response<VerifyProofResponse>, Status> {
    //     let req = request.into_inner();
    //     let proof = serde_json::to_string(&ProofResponse {
    //         gamma: req.gamma,
    //         c: req.c,
    //         s: req.s,
    //         beta: String::new(), // Not needed for verification
    //     })
    //     .map_err(|_| Status::internal("Failed to serialize proof"))?;

    //     match verify_proof(&req.public_key, &proof, &req.seed) {
    //         Ok(is_valid) => Ok(Response::new(VerifyProofResponse { is_valid })),
    //         Err(_) => Err(Status::internal("Failed to verify proof")),
    //     }
    // }
}

async fn serve_grpc(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = address.parse()?;
    let stark_vrf_service = StarkVrfService::default();

    println!("StarkVRF server listening on {}", addr);

    Server::builder()
        .add_service(StarkVrfServer::new(stark_vrf_service))
        .serve(addr)
        .await?;

    Ok(())
}
