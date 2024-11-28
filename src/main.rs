extern crate rand;
extern crate anyhow;
extern crate modular_math;
extern crate primitive_types;
extern crate pkcs8;

use std::{fmt::Display, str::FromStr};

use rand::{CryptoRng, Rng};
use anyhow::Result;
use modular_math::mod_math::ModMath;
use primitive_types::U256;
use pkcs8::{
    der::{
        asn1::{Null, OctetStringRef}, AnyRef, EncodePem
    }, spki::AlgorithmIdentifier, ObjectIdentifier, PrivateKeyInfo, SubjectPublicKeyInfo
};

const SECP256K1_OID: &str = "1.3.132.0.10";

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
struct Point {
    x: U256,
    y: U256,
}

#[derive(Clone, Debug)]
struct EllipticCurve {
    a: U256,
    b: U256,
}

struct EllipticConfig {
    name: String,
    oid: ObjectIdentifier,
    curve: EllipticCurve,
    prime: U256,
    mod_p: ModMath,
    base: Point,
    order: U256,
}

#[derive(Clone, Debug)]
struct EllipticKeys {
    config_name: String,
    config_oid: ObjectIdentifier,
    private: U256,
    public: Point,
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

impl EllipticConfig {
    fn new(name: String, oid: ObjectIdentifier, curve: EllipticCurve, prime: U256, base_x: U256, order: U256) -> Option<Self> {
        let mod_p = ModMath::new(prime);
        Some(Self {
            name,
            oid,
            base: Point::new_from_curve(base_x, &mod_p, &curve)?,
            curve,
            prime,
            order,
            mod_p,
        })
    }
}

impl Display for EllipticConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EllipticConfig:\n\tName:\n\t\t{}\n\tOID:\n\t\t{}\n\tCurve:\n\t\t{:?}\n\tPrime:\n\t\t{}\n\tBase:\n\t\t{:?}\n\tOrder:\n\t\t{}",
            self.name,
            self.oid,
            self.curve,
            self.prime,
            self.base,
            self.order
        )
    }
}

impl EllipticKeys {
    fn generate(config: EllipticConfig) -> Self {
        let mut rng = rand::thread_rng();

        let private = gen_u256_below(&mut rng, &config.order);
        let public = config.base.mult(&private, &config.mod_p, &config.curve);

        Self {
            config_name: config.name,
            config_oid: config.oid,
            private,
            public,
        }
    }

    #[deprecated(since = "0.0.1", note="This method is incomplete.")]
    fn pem_private(&self) -> String {
        let private_key_bytes: [u8; 32] = {
            let mut bytes = [0u8; 32];
            self.private.to_little_endian(&mut bytes);
            bytes
        };

        let public_key_x_bytes: [u8; 32] = {
            let mut bytes = [0u8; 32];
            self.public.x.to_little_endian(&mut bytes);
            bytes
        };

        let public_key_y_bytes: [u8; 32] = {
            let mut bytes = [0u8; 32];
            self.public.y.to_little_endian(&mut bytes);
            bytes
        };

        let mut full_public_key_bytes = [0u8; 64];
        full_public_key_bytes[..32].copy_from_slice(&public_key_x_bytes);
        full_public_key_bytes[32..].copy_from_slice(&public_key_y_bytes);

        let algorithm_ident: AlgorithmIdentifier<AnyRef<'_>> = AlgorithmIdentifier {
            oid: self.config_oid, // Use the curve-specific OID
            parameters: Some(Null.into()),
        };

        let private_key_octet_string = OctetStringRef::new(&private_key_bytes)
            .map_err(|e| format!("Failed to create octet string: {:?}", e)).unwrap();

        let private_key_info = PrivateKeyInfo {
            algorithm: algorithm_ident,
            private_key: private_key_octet_string.into(),
            public_key: Some(&full_public_key_bytes),
        };

        private_key_info.to_pem(pkcs8::LineEnding::CRLF).unwrap()
    }

    #[deprecated(since = "0.0.1", note="This method is incomplete.")]
    fn pem_public(&self) -> String {
        let public_key_x_bytes: [u8; 32] = {
            let mut bytes = [0u8; 32];
            self.public.x.to_little_endian(&mut bytes);
            bytes
        };

        let public_key_y_bytes: [u8; 32] = {
            let mut bytes = [0u8; 32];
            self.public.y.to_little_endian(&mut bytes);
            bytes
        };

        
        
        let mut full_public_key_bytes = [0u8; 64];
        full_public_key_bytes[..32].copy_from_slice(&public_key_x_bytes);
        full_public_key_bytes[32..].copy_from_slice(&public_key_y_bytes);
        
        let algorithm_ident: AlgorithmIdentifier<AnyRef<'_>> = AlgorithmIdentifier {
            oid: self.config_oid, // Use the curve-specific OID
            parameters: Some(Null.into()),
        };

        let public_key_info = SubjectPublicKeyInfo {
            algorithm: algorithm_ident,
            subject_public_key: full_public_key_bytes,
        };

        public_key_info.to_pem(pkcs8::LineEnding::CRLF).unwrap()
    }
}

impl Display for EllipticKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "EllipticKeys:\n\tConfig name:\n\t\t{}\n\tOID:\n\t\t{}\n\tPrivate:\n\t\t{}\n\tPublic:\n\t\t{:?}",
            self.config_name,
            self.config_oid,
            self.private,
            self.public
        )
    }
}

fn main() -> Result<()> {
    let secp256k1_curve = EllipticCurve {
        a: U256::from(0),
        b: U256::from(7),
    };
    let prime = U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F")?;
    let base_x = U256::from_str("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798")?;
    let order = U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141")?;
    let oid = ObjectIdentifier::from_str(SECP256K1_OID).unwrap();

    let secp256k1 = EllipticConfig::new("secp256k1".to_string(), oid, secp256k1_curve, prime, base_x, order).unwrap();
    println!("{}", secp256k1);

    let keys = EllipticKeys::generate(secp256k1);
    println!("{}", keys);

    println!("PEM output:\n\n{}\n\n{}", keys.pem_private(), keys.pem_public());

    Ok(())
}
