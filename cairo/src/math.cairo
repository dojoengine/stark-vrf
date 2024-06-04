use core::ec::stark_curve;
use starknet::testing::cheatcode;
use core::felt252_div;

// Sage script to find Z
//
// # Arguments:
// # - F, a field object, e.g., F = GF(2^521 - 1)
// # - A and B, the coefficients of the curve y^2 = x^3 + A * x + B
// def find_z_sswu(F, A, B):
//     R.<xx> = F[]                       # Polynomial ring over F
//     g = xx^3 + F(A) * xx + F(B)        # y^2 = g(x) = x^3 + A * x + B
//     ctr = F.gen()
//     while True:
//         for Z_cand in (F(ctr), F(-ctr)):
//             # Criterion 1: Z is non-square in F.
//             if is_square(Z_cand):
//                 continue
//             # Criterion 2: Z != -1 in F.
//             if Z_cand == F(-1):
//                 continue
//             # Criterion 3: g(x) - Z is irreducible over F.
//             if not (g - Z_cand).is_irreducible():
//                 continue
//             # Criterion 4: g(B / (Z * A)) is square in F.
//             if is_square(g(B / (Z_cand * A))):
//                 return Z_cand
//         ctr += 1

// find_z_sswu(FiniteField(P), stark_curve::ALPHA, stark_curve::BETA)
pub const Z: felt252 = 19;

// constants
pub const A: felt252 = stark_curve::ALPHA;
pub const B: felt252 = stark_curve::BETA;

// constants for sqrt_ratio
pub const c1: felt252 = 192;                // largest integer such that 2^c1 divides q - 1;
pub const c2: felt252 = 576460752303423505; // (q - 1) / 2^c1
pub const c3: u256 = 288230376151711752; // (c2 - 1) / 2
pub const c4: u256 = 6277101735386680763835789423207666416102355444464034512895; // 2^c1 - 1
pub const c5: u256 = 3138550867693340381917894711603833208051177722232017256448; // 2^(c1 - 1)
pub const c6: felt252 = 271122989245172633851355451947103605954274810690058238200515407597964208139; // Z^c2;
pub const c7: felt252 = 3564856802633786767843881147069258360232483436295672590729615622861023547560; // Z^((c2 + 1) / 2)


// Input: base and power, elements of F
// Output: base^power
pub fn fast_power(
    base: felt252, mut power: u256
) -> felt252 {
    base * base
    // assert!(base != 0_u8.into(), "fast_power: invalid input");

    // let mut base: felt252 = base.into();
    // let mut result: felt252 = 1;

    // loop {
    //     if power % 2_u8.into() != 0_u8.into() {
    //         result *= base;
    //     }
    //     power /= 2_u8.into();
    //     if (power == 0_u8.into()) {
    //         break;
    //     }
    //     base *= base;
    // };

    // result.try_into().expect('too large to fit output type')
}

// Input: u and v, elements of F, where v != 0.
// Output: (b, y), where
//   b = True and y = sqrt(u / v) if (u / v) is square in F, and
//   b = False and y = sqrt(Z * (u / v)) otherwise.
fn sqrt_ratio(u: felt252, v: felt252) -> (bool, felt252) {
    let result = cheatcode::<'sqrt_ratio'>(array![u, v, Z].span());
    println!("sqrt_ratio returned {result:?}");
    let result = *result.at(0);
    let div = felt252_div(u, v.try_into().unwrap());
    let result = if result * result == div {
        Option::Some((true, result))
    } else if result * result == Z * div {
        Option::Some((false, result))
    } else {
        Option::None
    };
    result.unwrap()

    // (true, v)

    // let mut tv1 = c6;
    // let tv2 = fast_power(v, c4);
    // let tv3 = tv2 * tv2;
    // let tv3 = tv3 * v;
    // let tv5 = u * tv3;
    // let tv5 = fast_power(tv5, c3);
    // let tv5 = tv5 * tv2;
    // let tv2 = tv5 * v;
    // let mut tv3 = tv5 * u;
    // let mut tv4 = tv3 * tv2;
    // let tv5 = fast_power(tv4, c5);
    // let isQR = tv5 == 1;
    // let mut e1 = tv5 == 1;
    // let mut tv2 = tv3 * c7;
    // let mut tv5 = tv4 * tv1;
    // if !isQR {
    //     tv3 = tv2;
    //     tv4 = tv5;
    // }
    // let mut i = c1;
    // loop {
    //     if i == 1 { break; }
    //     tv5 = i - 2;
    //     tv5 = fast_power(2, tv5.into());
    //     tv5 = fast_power(tv4, tv5.into());
    //     e1 = tv5 == 1;
    //     tv2 = tv3 * tv1;
    //     tv1 = tv1 * tv1;
    //     tv5 = tv4 * tv1;
    //     if !e1 {
    //         tv3 = tv2;
    //         tv4 = tv5;
    //     }

    //     i -= 1;
    // };
    // (isQR, tv3)
}
