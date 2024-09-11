pub mod curve;
pub mod ecvrf;
pub mod error;
pub mod hash;

use ark_ec::{
    short_weierstrass::{Affine, SWCurveConfig},
    CurveConfig, CurveGroup,
};
pub use ark_ff::MontFp as ScalarValue;
pub use curve::*;
pub use ecvrf::*;

pub type StarkVRF = ECVRF<StarkCurve, hash::PoseidonHash>;

pub fn generate_public_key(
    secret: <curve::StarkCurve as CurveConfig>::ScalarField,
) -> Affine<StarkCurve> {
    (StarkCurve::GENERATOR * secret).into_affine()
}

#[cfg(test)]
mod tests {
    use crate::{generate_public_key, ScalarValue, StarkVRF};

    #[test]
    fn it_proves_and_verifies() {
        let secret_key = ScalarValue!("190");
        let public_key = generate_public_key(secret_key);

        let seed = &[ScalarValue!("42")];
        let ecvrf = StarkVRF::new(public_key).unwrap();
        let proof = ecvrf.prove(&secret_key, seed).unwrap();
        let sqrt_ratio_hint = ecvrf.hash_to_sqrt_ratio_hint(seed);
        let beta = ecvrf.proof_to_hash(&proof).unwrap();

        println!("public key: {public_key:?}");
        println!("seed: 42");
        println!("proof gamma: {}", proof.0);
        println!("proof c: {}", proof.1);
        println!("proof s: {}", proof.2);
        println!("proof verify hint: {}", sqrt_ratio_hint);

        ecvrf.verify(&proof, seed).expect("proof correct");
        println!("proof verified, random value = {beta}");
    }
}
