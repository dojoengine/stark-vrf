use ark_ec::{
    hashing::{
        curve_maps::swu::{SWUConfig, SWUMap},
        map_to_curve_hasher::MapToCurve,
    },
    short_weierstrass::{Affine, SWCurveConfig},
    AffineRepr, CurveConfig,
};
use ark_ff::BigInt;

use crate::error::Error;
use crate::{error::Result, hash::HashToField};

pub type Proof<Curve> = (
    Affine<Curve>,
    <Curve as CurveConfig>::ScalarField,
    <Curve as CurveConfig>::ScalarField,
);

pub struct ECVRF<Curve, Hasher>
where
    Curve: SWCurveConfig + SWUConfig,
    Curve::BaseField: From<BigInt<4>> + Into<BigInt<4>>,
    Curve::ScalarField: From<BigInt<4>> + Into<BigInt<4>>,
    Hasher: HashToField<Curve>,
{
    public_key: Affine<Curve>,
    pub mapper: SWUMap<Curve>,
    hasher: Hasher,
}

impl<Curve, Hasher> ECVRF<Curve, Hasher>
where
    Curve: SWCurveConfig + SWUConfig,
    Curve::BaseField: From<BigInt<4>> + Into<BigInt<4>>,
    Curve::ScalarField: From<BigInt<4>> + Into<BigInt<4>>,
    Hasher: HashToField<Curve>,
{
    pub fn new(public_key: Affine<Curve>) -> Result<Self> {
        Ok(Self {
            public_key,
            mapper: SWUMap::new()?,
            hasher: Hasher::new(),
        })
    }

    pub fn prove(&self, secret_key: &Curve::ScalarField, seed: &[Curve::BaseField]) -> Result<Proof<Curve>> {
        let pk_from_secret = Curve::GENERATOR * secret_key;
        if self.public_key != pk_from_secret {
            return Err(Error::InvalidSecretKey);
        }

        let h = self.hash_to_curve(seed)?;

        let gamma: Affine<Curve> = (h * secret_key).into();
        let k = self.nonce(secret_key, seed)?;
        let c = self.hash_points(&[
            self.public_key,
            h,
            gamma,
            (Curve::GENERATOR * k).into(),
            (h * k).into(),
        ])?;
        let s = k + c * secret_key;
        Ok((gamma, c, s))
    }

    pub fn proof_to_hash(&self, proof: &Proof<Curve>) -> Result<Curve::BaseField> {

        let mut cofactor_buf: [u64; 4] = [0; 4];
        for (i, limb) in Curve::COFACTOR.iter().enumerate() {
            cofactor_buf[i] = *limb;
        }

        let cofactor_gamma = proof.0.mul_by_cofactor_to_group();
        // our cofactor is 1
        assert_eq!(proof.0, cofactor_gamma);

        let string: Vec<Curve::BaseField> = vec![
            BigInt!("3").into(),
            proof.0.x,
            proof.0.y,
            BigInt!("0").into(),
        ];
        Ok(self.hasher.hash_to_base(&string))
    }

    pub fn verify(&self, proof: &Proof<Curve>, seed: &[Curve::BaseField]) -> Result<()> {
        let pk = self.public_key;
        let (gamma, c, s) = proof;

        println!("verify RS gamma: {}", gamma);
        println!("verify RS c: {}", c);
        println!("verify RS s: {}", s);

        let h = self.hash_to_curve(seed)?;

        println!("verify RS h {}", h);
        let u = (Curve::GENERATOR * s) - (pk * *c);
        println!("verify RS u {}", u);

        let v = (h * s) - (*gamma * *c);
        println!("verify RS v {}", v);

        let c_prim = self.hash_points(&[pk, h, *gamma, u.into(), v.into()])?;
        println!("verify RS c_prim {}", c_prim);

        if *c == c_prim {
            Ok(())
        } else {
            Err(Error::ProofVerificationError)
        }
    }

    fn hash_to_curve(&self, message: &[Curve::BaseField]) -> Result<Affine<Curve>> {
        let pk = self.public_key;
        let mut buf: Vec<Curve::BaseField> = Vec::new();
        buf.push(pk.x);
        buf.push(pk.y);
        buf.push(BigInt!("1").into());
        buf.extend_from_slice(message);

        for el in &buf {
            println!("hash_to_curve input {el}");
        }
        let t = self.hasher.hash_to_base(&buf);
        println!("hash_to_curve output {t}");

        Ok(self.mapper.map_to_curve(t)?)
    }

    fn hash_points(&self, points: &[Affine<Curve>]) -> Result<Curve::ScalarField> {
        let mut string = vec![BigInt!("2").into()];
        for point in points {
            string.push(point.x);
            string.push(point.y);
        }
        string.extend_from_slice(&[BigInt!("0").into()]);

        // TODO: typically challenges have half the number of bits of the
        // scalar field.
        // for now this returns a full scalar field value
        let fr = self.hasher.hash_to_scalar(&string);
        Ok(fr)
    }

    // 5.4.2.2. ECVRF Nonce Generation from RFC 8032
    pub fn nonce(
        &self,
        secret_key: &Curve::ScalarField,
        seed: &[Curve::BaseField],
    ) -> Result<Curve::ScalarField> {
        let base_sk = *secret_key;
        let sk = self.mapper.map_to_curve(Curve::BaseField::from(base_sk.into()))?;
        let mut buf = vec![sk.x, sk.y];
        buf.extend_from_slice(seed);
        let fr = self.hasher.hash_to_scalar(buf.as_slice());
        Ok(fr)
    }
}
