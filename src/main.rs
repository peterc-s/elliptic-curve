extern crate num_bigint;
extern crate rand;
extern crate anyhow;
extern crate modular_math;

use std::str::FromStr;

use rand::{CryptoRng, Rng};
use anyhow::Result;
use modular_math::mod_math::ModMath;
use primitive_types::U256;

fn gen_u256_below<T: Rng + CryptoRng>(rng: &mut T, n: &U256) -> U256 {
    loop {
        let random_bytes: [u8; 32] = rng.gen();
        let random_u256: U256 = U256::from_little_endian(&random_bytes);

        if random_u256 < *n && random_u256 > U256::zero() {
            return random_u256;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
struct Point {
    x: U256,
    y: U256,
}

#[derive(Clone, Debug)]
struct EllipticCurve {
    a: U256,
    b: U256,
}

#[allow(dead_code)]
impl Point {
    fn new(x: U256, y: U256) -> Self {
        Point { x, y }
    }

    fn new_from_curve(x: U256, p: &ModMath, curve: &EllipticCurve) -> Option<Self> {
        let y = p.sqrt(p.exp(x, U256::from(3)) + curve.a * x + curve.b)?;

        Some(Point { x, y })
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
        let x: U256 = p.sub(p.sub(p.square(lambda), other.x), self.x);

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
            let numerator = p.mul(U256::from(3), p.square(self.x));
            // 2y
            let denominator = p.mul(U256::from(2), self.y);
            // (3x^2 + a) / 2y
            p.div(p.add(numerator, curve.a), denominator)
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
    let mut rng = rand::thread_rng();

    let secp256k1_curve = EllipticCurve {
        a: U256::from(0),
        b: U256::from(7),
    };
    println!("Curve is: {:?}", secp256k1_curve);

    let p = U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F")?;
    let mod_p = ModMath::new(p);
    println!("Prime is: {}", p);

    let base_x = U256::from_str("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798")?;
    println!("Base x: {}", base_x);

    let base_point = Point::new_from_curve(base_x, &mod_p, &secp256k1_curve).unwrap();
    println!("Base point is {:?}", base_point);

    let order = U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141")?;
    println!("Order is: {:?}", order);

    let priv_key = gen_u256_below(&mut rng, &order);
    println!("Private key: {:?}", priv_key);

    let pub_key = base_point.mult(&priv_key, &mod_p, &secp256k1_curve);
    println!("Public key: {:?}", pub_key);

    Ok(())
}
