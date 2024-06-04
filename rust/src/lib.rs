mod curve;
mod ecvrf;
pub mod error;
pub mod hash;

pub use curve::*;
pub use ecvrf::*;

pub type StarkVRF = ECVRF::<StarkCurve, hash::PoseidonHash>;

#[cfg(test)]
mod tests {
    use ark_ec::{
        short_weierstrass::{Affine, SWCurveConfig}, CurveConfig, CurveGroup
    };
    use ark_ec::hashing::map_to_curve_hasher::MapToCurve;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use starknet_ff::FieldElement;
    use ark_ff::{BigInt, BigInteger, MontFp, PrimeField};


    use crate::{
        curve::{BaseField, ScalarField, StarkCurve},
        hash::PoseidonHash,
        StarkVRF,
    };

    #[test]
    fn point_serialization() {
        let mut buf = Vec::new();

        let g = StarkCurve::GENERATOR;
        assert!(g.is_on_curve());
        assert!(g.is_in_correct_subgroup_assuming_on_curve());

        g.serialize_compressed(&mut buf).unwrap();
        let deg = Affine::<StarkCurve>::deserialize_compressed(&*buf).unwrap();
        assert_eq!(g, deg);
    }

    #[test]
    fn field_element_conversion() {
        let buf1 = [
            FieldElement::from_dec_str(
                "2465182048640915825114623967805639036884813714770257338089158027381626459289",
            )
            .unwrap(),
            FieldElement::from_dec_str(
                "3038635738014387716559859267483610492356329532552881764846792983975787300333",
            )
            .unwrap(),
            FieldElement::from_dec_str("1").unwrap(),
            FieldElement::from_dec_str("42").unwrap(),
        ];

    }
    #[test]
    fn it_matches_cairo_hashing() {
        use starknet_crypto::poseidon_hash_many;

        let buf = [
            FieldElement::from_dec_str(
                "2465182048640915825114623967805639036884813714770257338089158027381626459289",
            )
            .unwrap(),
            FieldElement::from_dec_str(
                "3038635738014387716559859267483610492356329532552881764846792983975787300333",
            )
            .unwrap(),
            FieldElement::from_dec_str("1").unwrap(),
            FieldElement::from_dec_str("42").unwrap(),
        ];

        let g = StarkCurve::GENERATOR;
        let secret_key = ScalarField::from(190);
        let public_key = (g * secret_key).into_affine();
        let ecvrf = StarkVRF::new(public_key).unwrap();

        let hash = poseidon_hash_many(buf.as_slice());
        let hash_in_base = BaseField::new_unchecked(BigInt(hash.into_mont()));

        let point = ecvrf.mapper.map_to_curve(hash_in_base).unwrap();
        println!("CAIRO hash {hash} {hash_in_base}");
        println!("point {point}");
    }

    #[test]
    fn it_proves() {
        let secret_key = ScalarField::from(190);
        let public_key = (StarkCurve::GENERATOR * secret_key).into_affine();

        let seed = &[MontFp!("42")];
        let ecvrf = StarkVRF::new(public_key).unwrap();
        let proof = ecvrf.prove(&secret_key, seed).unwrap();
        let beta = ecvrf.proof_to_hash(&proof).unwrap();

        println!("public key: {public_key:?}");
        println!("seed: 42");
        println!("proof gamma: {}", proof.0);
        println!("proof c: {}", proof.1);
        println!("proof s: {}", proof.2);


        ecvrf.verify(&proof, seed).expect("proof correct");
        println!("proof verified, beta = {beta}");
    }

    #[test]
    fn felt_representation() {
        let a = FieldElement::from_dec_str(
            "2465182048640915825114623967805639036884813714770257338089158027381626459289",
        )
        .unwrap();
        println!("FieldElement {a}");

        let b: <StarkCurve as CurveConfig>::BaseField = MontFp!("2465182048640915825114623967805639036884813714770257338089158027381626459289");
        println!("BaseField {b}");

        let bytes = a.to_bits_le();

        let c = <StarkCurve as CurveConfig>::BaseField::from_bigint(BigInt::from_bits_le(&bytes)).unwrap();
        println!("BaseField B {c}");

    }
}
