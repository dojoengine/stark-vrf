pub mod ecvrf;
pub mod error;
mod math;

pub use ecvrf::{Point, Proof, ECVRF, ECVRFImpl};

#[cfg(test)]
mod tests {
    use core::option::OptionTrait;
    use super::ecvrf::{hash_to_curve, Proof, Point, ECVRFImpl};

    fn proof_from_oracle() -> Proof {
        Proof {
            gamma: Point {
                x: 1506339363762384048749124975867331702319430609263271304275332020910807468800,
                y: 36259598506905210600179635686591002688831785399437338349196739602416217657
            },
            c: 2613903846701008054856365693011070443633034612733309583190565217827378733393,
            s: 1867682679224997956048283066055885717352683300581532690215097247223135564277,
        }
    }
    
    #[test]
    fn ecvrf_verify() {
        let pk = Point {
            x: 2465182048640915825114623967805639036884813714770257338089158027381626459289,
            y: 3038635738014387716559859267483610492356329532552881764846792983975787300333
        }; 
        let proof = proof_from_oracle();
        let ecvrf = ECVRFImpl::new(pk);
        let mut seed = ArrayTrait::new();
        seed.append(42);
    
        let expected_beta = 1749760720107131022781690892024891617311129198096286233628341005792224087740;
        let actual_beta = ecvrf.verify(proof, seed.span()).unwrap();
        assert_eq!(expected_beta, actual_beta);
    }
}
