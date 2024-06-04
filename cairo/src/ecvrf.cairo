use core::array::SpanTrait;
use core::option::OptionTrait;
use core::traits::TryInto;

use core::ec::{EcPointImpl, NonZeroEcPoint, EcPointTryIntoNonZero, EcPoint, stark_curve, ec_point_unwrap};
use core::num::traits::zero::Zero;
use core::poseidon::poseidon_hash_span;
use super::math::{Z, A, B, sqrt_ratio};
use super::error::Error;

pub extern fn felt252_div(lhs: felt252, rhs: NonZero<felt252>) -> felt252 nopanic;

#[derive(Copy, Drop, Serde)]
pub struct Point {
    pub x: felt252,
    pub y: felt252,
}

#[derive(Clone, Drop, Serde)]
pub struct Proof {
    gamma: Point,
    c: felt252,
    s: felt252,
}

#[derive(Drop)]
pub struct ECVRF {
    pub pk: Point,
    g: EcPoint,
}

#[generate_trait]
pub impl ECVRFImpl of ECVRFTrait {
    fn new(pk: Point) -> ECVRF {
        ECVRF {
            pk,
            g: EcPointImpl::new(stark_curve::GEN_X, stark_curve::GEN_Y).unwrap(),
        }
    }

    fn verify(self: @ECVRF, proof: Proof, seed: Span<felt252>) -> Result<felt252, Error> {
        let Proof { gamma, c, s} = proof.clone();
        let Point { x, y } = gamma;
        // println!("verify Cairo gamma {x} {y}");
        // println!("verify Cairo c {c}");
        // println!("verify Cairo s {s}");

        let pk = *self.pk;
        let ec_pk = EcPointImpl::new(pk.x, pk.y).unwrap();

        let g = *self.g;
        let h = hash_to_curve(pk, seed)?;
        let (gx, gy) = ec_point_unwrap(h.try_into().unwrap());
        // println!("verify Cairo h {gx} {gy}");
    
        let u = g.mul(s) - ec_pk.mul(c);
        let (gx, gy) = ec_point_unwrap(u.try_into().unwrap());
        // println!("verify Cairo u {gx} {gy}");

        let gamma = EcPointImpl::new(x, y).unwrap();

        let v = h.mul(s) - gamma.mul(c);
        let (gx, gy) = ec_point_unwrap(v.try_into().unwrap());
        // println!("verify Cairo v {gx} {gy}");

        
        let mut challenge = ArrayTrait::new();
        challenge.append(2);
        let Point { x, y } = pk;
        challenge.append(x);
        challenge.append(y);
        let (x, y) = ec_point_unwrap(h.try_into().unwrap());
        challenge.append(x);
        challenge.append(y);
        let (x, y) = ec_point_unwrap(gamma.try_into().unwrap());
        challenge.append(x);
        challenge.append(y);
        let (x, y) = ec_point_unwrap(u.try_into().unwrap());
        challenge.append(x);
        challenge.append(y);
        let (x, y) = ec_point_unwrap(v.try_into().unwrap());
        challenge.append(x);
        challenge.append(y);
        challenge.append(0);
        let c_prim = poseidon_hash_span(challenge.span());

        if c == c_prim {
            self.proof_to_hash(proof)
        } else {
            Result::Err(Error::ProofVerificationError)
        }
    }

    fn proof_to_hash(self: @ECVRF, proof: Proof) -> Result<felt252, Error> {
        // cofactor is 1, we can simply use gamma
        let mut beta = ArrayTrait::new();
        beta.append(3);
        let Point { x, y } = proof.gamma;
        beta.append(x);
        beta.append(y);
        beta.append(0);
        Result::Ok(poseidon_hash_span(beta.span()))
    }
}

pub fn hash_to_curve(pk: Point, a: Span<felt252>) -> Result<EcPoint, Error> {
    let Point { x, y } = pk;

    let mut buf = ArrayTrait::new();
    buf.append(x);
    buf.append(y);
    buf.append(1);
    buf.append_span(a);

    let mut hash = poseidon_hash_span(buf.span());
    // println!("buf: {buf:?} hash: {hash}");
    
    map_to_curve(hash)
}

// map_to_curve_simple_swu(u)
//   Input: u, an element of F.
//   Output: (x, y), a point on E.
fn map_to_curve(u: felt252) -> Result<EcPoint, Error> {
    let tv1 = Z * u * u;
    let tv2 = tv1 * tv1 + tv1;
    let tv3 = B * (tv2 + 1);
    let tv4 = if tv2.is_zero() {
        Z
    } else {
        -tv2
    };
    let tv4 = A * tv4;
    let tv2 = tv3 * tv3;
    let tv6 = tv4 * tv4;
    let tv5 = A * tv6;
    let tv2 = tv2 + tv5;
    let tv2 = tv2 * tv3;
    let tv6 = tv6 * tv4;
    let tv5 = B * tv6;
    let tv2 = tv2 + tv5;
    let x = tv1 * tv3;
    let (is_gx1_square, y1) = sqrt_ratio(tv2, tv6);
    let y = tv1 * u;
    let y = y * y1;
    let (x, y) = if is_gx1_square {
        (tv3, y1)
    } else {
        (x, y)
    };

    let u_256: u256 = u.into();
    let y_256: u256 = y.into();
    let y = if (u_256 % 2) == (y_256 % 2) {
        y
    } else {
        -y
    };

    let x = felt252_div(x, tv4.try_into().unwrap());
    Result::Ok(EcPointImpl::new(x, y).unwrap())
}
