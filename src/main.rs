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
            return self.double(curve);
        }
        if *self == Self::identity() {
            return self.clone();
        }
        if *other == Self::identity() {
            return other.clone();
        }

        let lambda = {
            // y2 - y1 / x2 - x1
            let numerator = p.sub(other.y, self.y);
            let denominator = p.sub(other.x, self.x);
            p.mul(numerator, p.inv(denominator).unwrap())
        };

        // x3 = \lambda^2 - x2 - x1
        let x: U256 = p.sub(p.sub(p.exp(lambda, U256::from(2)), other.x), self.x);

        // \lambda(x2 - x3)-y2
        let y: U256 = p.sub(p.mul(lambda, p.sub(other.x, x)), other.y);

        Point { x, y }
    }

    fn double(&self, curve: &EllipticCurve) -> Self {
        todo!()
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

    println!("Curve point 1: {}, {}", curve_point_1.x, curve_point_1.y);
    println!("Curve point 2: {}, {}", curve_point_2.x, curve_point_2.y);

    let curve_points_added = curve_point_1.add_curve(&curve_point_2, &mod_p, &test_curve);

    println!("Curve points added: {}, {}", curve_points_added.x, curve_points_added.y);

    Ok(())
}
