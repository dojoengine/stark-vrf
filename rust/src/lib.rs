mod curve;
mod ecvrf;
pub mod error;
pub mod hash;

pub use curve::*;
pub use ecvrf::*;

pub type StarkVRF = ECVRF<StarkCurve, hash::PoseidonHash>;

#[cfg(test)]
mod tests {
    use ark_ec::{short_weierstrass::SWCurveConfig, CurveGroup};
    use ark_ff::MontFp;

    use crate::{
        curve::{ScalarField, StarkCurve},
        StarkVRF,
    };

    #[test]
    fn it_proves_and_verifies() {
        let secret_key = ScalarField::from(190);
        let public_key = (StarkCurve::GENERATOR * secret_key).into_affine();

        let seed = &[MontFp!("42")];
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
