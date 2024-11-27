extern crate num_bigint;
extern crate rand;
extern crate anyhow;
extern crate modular_math;

use num_bigint::{BigUint, RandomBits};
use rand::{CryptoRng, Rng};
use anyhow::Result;
use modular_math::mod_math::ModMath;
use primitive_types::U256;

fn gen_biguint256<T: Rng + CryptoRng>(rng: &mut T) -> BigUint {
    return rng.sample(RandomBits::new(256))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
struct Point {
    x: U256,
    y: U256,
}

#[derive(Clone, Debug)]
struct EllipticCurve {
    a: i32,
    b: i32,
}

#[allow(dead_code)]
impl Point {
    fn new(x: U256, y: U256) -> Self {
        Point { x, y }
    }

    fn new_from_curve(x: U256, p: &ModMath, curve: &EllipticCurve) -> Self {
        let y = p.sqrt(x.pow(U256::from(3)) + U256::from(curve.a) * x + U256::from(curve.b)).unwrap();

        Point { x, y }
    }

    fn identity() -> Self {
        Point {
            x: U256::zero(),
            y: U256::zero(),
        }
    }

    fn add_curve(&self, other: &Point, p: &ModMath, curve: &EllipticCurve) -> Self {
        if *self == *other || self.x == other.x {
            return self.double(p, curve);
        }
        if *self == Self::identity() {
            return other.clone();
        }
        if *other == Self::identity() {
            return self.clone();
        }

        let lambda = {
            // y2 - y1 / x2 - x1
            let numerator = p.sub(other.y, self.y);
            let denominator = p.sub(other.x, self.x);
            p.div(numerator, denominator)
        };

        // x3 = \lambda^2 - x2 - x1
        let x: U256 = p.sub(p.sub(p.exp(lambda, U256::from(2)), other.x), self.x);

        // \lambda(x2 - x3)-y2
        let y: U256 = p.sub(p.mul(lambda, p.sub(other.x, x)), other.y);

        Point { x, y }
    }

    fn double(&self, p: &ModMath, curve: &EllipticCurve) -> Self {
        if *self == Self::identity() {
            return self.clone();
        }

        let lambda = {
            // 3x^2
            let numerator = p.mul(U256::from(3), p.exp(self.x, U256::from(2)));
            // 2y
            let denominator = p.mul(U256::from(2), self.y);
            // (3x^2 + a) / 2y
            p.div(p.add(numerator, U256::from(curve.a)), denominator)
        };

        // x' = \lambda^2 - 2x
        let x: U256 = p.sub(p.exp(lambda, U256::from(2)), p.mul(U256::from(2), self.x));
        // y' = \lambda(x - x') - y
        let y: U256 = p.sub(p.mul(lambda, p.sub(self.x, x)), self.y);

        Point { x, y }
    }

    fn mult(&self, k: &U256, p: &ModMath, curve: &EllipticCurve) -> Self {
        let mut result = Self::identity();
        let mut current = self.clone();

        let mut k = k.clone();

        while k > U256::zero() {
            if k % U256::from(2) == U256::one() {
                result = result.add_curve(&current, p, curve);
            }
            current = current.double(p, curve);
            k = k >> 1;
        }

        result
    }
}

fn main() -> Result<()> {
    // let p: U256 = U256::MAX - U256::from(189u8);
    let p = "97";

    let mod_p = ModMath::new(p);

    println!("Prime is: {}", p);

    let test_curve = EllipticCurve {
        a: 2,
        b: 3,
    };

    // let secp256k1_curve = EllipticCurve {
    //     a: 0,
    //     b: 7,
    // };

    let curve_point_1 = Point::new_from_curve(U256::from(17u8), &mod_p, &test_curve);
    let curve_point_2 = Point::new_from_curve(U256::from(95u8), &mod_p, &test_curve);

    println!("Curve point 1: {:?}", curve_point_1);
    println!("Curve point 2: {:?}", curve_point_2);

    let curve_points_added = curve_point_1.add_curve(&curve_point_2, &mod_p, &test_curve);

    println!("Curve points added: {:?}", curve_points_added);
    println!("First point doubled: {:?}", curve_point_1.double(&mod_p, &test_curve));
    println!("First point 7P: {:?}", curve_point_1.mult(&U256::from(7), &mod_p, &test_curve));

    Ok(())
}
