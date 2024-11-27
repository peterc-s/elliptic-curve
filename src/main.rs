extern crate num_bigint;
extern crate rand;
extern crate anyhow;


use num_bigint::{BigInt, BigUint, ToBigInt, RandomBits};
use rand::{CryptoRng, Rng};
use anyhow::Result;

fn gen_biguint256<T: Rng + CryptoRng>(rng: &mut T) -> BigUint {
    return rng.sample(RandomBits::new(256))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
struct Point {
    x: BigUint,
    y: BigUint,
}

#[derive(Clone, Debug)]
struct EllipticCurve {
    a: i32,
    b: i32,
}

#[allow(dead_code)]
impl Point {
    fn new(x: BigUint, y: BigUint) -> Self {
        Point { x, y }
    }

    fn identity() -> Self {
        Point {
            x: BigUint::ZERO,
            y: BigUint::ZERO,
        }
    }

    fn add(&self, other: &Point, curve: &EllipticCurve) -> Self {
        if *self == *other {
            return self.double(curve);
        }
        if *self == Self::identity() {
            return self.clone();
        }
        if *other == Self::identity() {
            return other.clone();
        }

        todo!()
    }

    fn double(&self, curve: &EllipticCurve) -> Self {
        todo!()
    }
}

fn mod_add(a: &BigUint, b: &BigUint, p: &BigUint) -> BigUint {
    ((a + b) % p + p) % p
}

fn mod_sub(a: &BigUint, b: &BigUint, p: &BigUint) -> BigUint {
    // convert to signed incase a - b is negative, avoids wrapping.
    let a_signed: &BigInt = &a.to_bigint().expect(format!("Unable to convert a: {} to BigInt.", a).as_str());
    let b_signed: &BigInt = &b.to_bigint().expect(format!("Unable to convert b: {} to BigInt.", b).as_str());
    let p_signed: &BigInt = &p.to_bigint().expect(format!("Unable to convert p: {} to BigInt.", p).as_str());

    // modulo operation rather than remainder.
    let result_signed: BigInt = ((a_signed - b_signed) % p_signed + p_signed) % p_signed;

    // convert result back to uint, provided it is now positive
    let result: BigUint = result_signed.to_biguint().expect(format!("Unable to convert result: {} to BigUint", result_signed).as_str());

    result
}

fn mod_mul(a: &BigUint, b: &BigUint, p: &BigUint) -> BigUint {
    ((a * b) % p + p) % p
}

fn mod_inv(a: &BigUint, p: &BigUint) -> BigUint {
    a.modinv(p).expect(format!("Unable to find modular inverse of {} given modulus {}.", a, p).as_str())
}

fn mod_div(a: &BigUint, b: &BigUint, p: &BigUint) -> BigUint {
    mod_mul(a, &mod_inv(b, p), p)
}


fn main() -> Result<()> {
    let mut rng = rand::thread_rng();
    let a: BigUint = gen_biguint256(&mut rng);
    let b: BigUint = gen_biguint256(&mut rng);
    let p: BigUint = BigUint::from(2u8).pow(256) - BigUint::from(189u8);

    let secp256k1_curve = EllipticCurve {
        a: 0,
        b: 7,
    };

    let sub: BigUint = mod_sub(&a, &b, &p);
    let add: BigUint = mod_add(&a, &b, &p);
    let mul: BigUint = mod_mul(&a, &b, &p);
    let inv: BigUint = mod_inv(&a, &p);

    println!("{}-{} (mod {}) = {}", a, b, p, sub);
    println!("{}+{} (mod {}) = {}", a, b, p, add);
    println!("{}*{} (mod {}) = {}", a, b, p, mul);
    println!("mod inv {} given {}: {}", a, p, inv);

    Ok(())
}
