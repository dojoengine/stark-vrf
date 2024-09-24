mod utils;

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use utils::set_panic_hook;

use wasm_bindgen::prelude::*;

use ark_ec::{
    short_weierstrass::{Affine, SWCurveConfig},
    AffineRepr, CurveGroup,
};
use starknet_ff::FieldElement;

use stark_vrf::{
    base_field_from_field_element, scalar_field_from_field_element, Proof, StarkCurve, StarkVRF,
};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(js_name = StarkVRF)]
pub struct StarkVRFJs {
    inner: StarkVRF,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ProofJs {
    gamma: Point,
    c: String,
    s: String,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Point {
    x: String,
    y: String,
}

impl From<&Proof<StarkCurve>> for ProofJs {
    fn from(value: &Proof<StarkCurve>) -> Self {
        ProofJs {
            gamma: Point {
                x: value.0.x().unwrap().to_string(),
                y: value.0.y().unwrap().to_string(),
            },
            c: value.1.to_string(),
            s: value.2.to_string(),
        }
    }
}

impl From<ProofJs> for Proof<StarkCurve> {
    fn from(value: ProofJs) -> Self {
        let x = FieldElement::from_dec_str(&value.gamma.x).unwrap();
        let y = FieldElement::from_dec_str(&value.gamma.y).unwrap();
        let c = FieldElement::from_dec_str(&value.c).unwrap();
        let s = FieldElement::from_dec_str(&value.s).unwrap();

        (
            Affine::<StarkCurve>::new(
                base_field_from_field_element(&x),
                base_field_from_field_element(&y),
            ),
            scalar_field_from_field_element(&c),
            scalar_field_from_field_element(&s),
        )
    }
}

#[wasm_bindgen(js_class = StarkVRF)]
impl StarkVRFJs {
    pub fn new(secret_key: String) -> Self {
        set_panic_hook();

        let secret_key = FieldElement::from_hex_be(&secret_key).unwrap();
        let secret_key = scalar_field_from_field_element(&secret_key);

        let public_key = (StarkCurve::GENERATOR * secret_key).into_affine();

        let ecvrf = StarkVRF::new(public_key).unwrap();

        Self { inner: ecvrf }
    }

    #[wasm_bindgen]
    pub fn prove(&self, secret_key: String, seed: String) -> ProofJs {
        set_panic_hook();

        let secret_key = FieldElement::from_hex_be(&secret_key).unwrap();
        let secret_key = scalar_field_from_field_element(&secret_key);

        let seed = FieldElement::from_hex_be(&seed).unwrap();
        let seed = base_field_from_field_element(&seed);

        let proof: Proof<StarkCurve> = self.inner.prove(&secret_key, &[seed]).unwrap();

        ProofJs::from(&proof)
    }

    #[wasm_bindgen(js_name = hashToSqrtRatioHint)]
    pub fn hash_to_sqrt_ratio_hint(&self, seed: String) -> String {
        set_panic_hook();

        let seed = FieldElement::from_hex_be(&seed).unwrap();
        let seed = base_field_from_field_element(&seed);

        self.inner.hash_to_sqrt_ratio_hint(&[seed]).to_string()
    }

    #[wasm_bindgen(js_name = proofToHash)]
    pub fn proof_to_hash(&self, proof: ProofJs) -> String {
        set_panic_hook();

        let proof: Proof<StarkCurve> = proof.into();
        self.inner.proof_to_hash(&proof).unwrap().to_string()
    }

    #[wasm_bindgen]
    pub fn verify(&self, proof: ProofJs, seed: String) -> bool {
        set_panic_hook();

        let seed = FieldElement::from_hex_be(&seed).unwrap();
        let seed = base_field_from_field_element(&seed);

        let proof: Proof<StarkCurve> = proof.into();

        match self.inner.verify(&proof, &[seed]) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
