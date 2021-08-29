use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_TABLE;
use std::ops::{Add, Mul};
use crate::cryptosystems::DecryptDirectly;
use scicrypt_traits::cryptosystems::AsymmetricCryptosystem;
use scicrypt_traits::security::BitsOfSecurity;
use scicrypt_traits::randomness::SecureRng;
use scicrypt_traits::Enrichable;

/// ElGamal over the Ristretto-encoded Curve25519 elliptic curve. The curve is provided by the
/// `curve25519-dalek` crate. ElGamal is a partially homomorphic cryptosystem.
pub struct CurveElGamal;

/// ElGamal ciphertext containing curve points. The addition operator on the ciphertext is
/// reflected as the curve operation on the associated plaintext.
#[derive(Debug, PartialEq)]
pub struct CurveElGamalCiphertext {
    pub(crate) c1: RistrettoPoint,
    pub(crate) c2: RistrettoPoint,
}

/// A struct holding both a ciphertext and a reference to its associated public key, which is
/// useful for decrypting directly using the secret key or performing homomorphic operations.
pub struct RichCurveElGamalCiphertext<'pk> {
    /// The ciphertext to operate on
    pub ciphertext: CurveElGamalCiphertext,
    /// Reference to the associated public key
    pub public_key: &'pk RistrettoPoint,
}

impl<'pk> Enrichable<'pk, RistrettoPoint, RichCurveElGamalCiphertext<'pk>> for CurveElGamalCiphertext {
    fn enrich(self, public_key: &RistrettoPoint) -> RichCurveElGamalCiphertext where Self: Sized {
        RichCurveElGamalCiphertext {
            ciphertext: self,
            public_key,
        }
    }
}

impl DecryptDirectly for CurveElGamal {
    type Plaintext = RistrettoPoint;
    type Ciphertext = CurveElGamalCiphertext;

    type SecretKey = Scalar;

    fn decrypt_direct(
        ciphertext: &Self::Ciphertext,
        secret_key: &Self::SecretKey,
    ) -> Self::Plaintext {
        ciphertext.c2 - secret_key * ciphertext.c1
    }
}

impl AsymmetricCryptosystem<'_> for CurveElGamal {
    type Plaintext = RistrettoPoint;
    type Ciphertext = CurveElGamalCiphertext;
    type RichCiphertext<'pk> = RichCurveElGamalCiphertext<'pk>;

    type PublicKey = RistrettoPoint;
    type SecretKey = Scalar;

    fn generate_keys<R: rand_core::RngCore + rand_core::CryptoRng>(
        security_param: &BitsOfSecurity,
        rng: &mut SecureRng<R>,
    ) -> (Self::PublicKey, Self::SecretKey) {
        match security_param {
            BitsOfSecurity::AES128 => (),
            _ => panic!(
                "Currently only the Ristretto group is supported with security level AES128."
            ),
        }

        let secret_key = Scalar::random(rng.rng());
        let public_key = &secret_key * &RISTRETTO_BASEPOINT_TABLE;

        (public_key, secret_key)
    }

    fn encrypt<R: rand_core::RngCore + rand_core::CryptoRng>(
        plaintext: &Self::Plaintext,
        public_key: &Self::PublicKey,
        rng: &mut SecureRng<R>,
    ) -> Self::Ciphertext {
        let y = Scalar::random(rng.rng());

        CurveElGamalCiphertext {
            c1: &y * &RISTRETTO_BASEPOINT_TABLE,
            c2: plaintext + y * public_key,
        }
    }

    fn decrypt(
        rich_ciphertext: &RichCurveElGamalCiphertext,
        secret_key: &Self::SecretKey,
    ) -> Self::Plaintext {
        Self::decrypt_direct(&rich_ciphertext.ciphertext, secret_key)
    }
}

impl Add for &CurveElGamalCiphertext {
    type Output = CurveElGamalCiphertext;

    /// Homomorphic operation between two ElGamal ciphertexts.
    fn add(self, rhs: Self) -> Self::Output {
        CurveElGamalCiphertext {
            c1: self.c1 + rhs.c1,
            c2: self.c2 + rhs.c2,
        }
    }
}

impl Mul<&Scalar> for &CurveElGamalCiphertext {
    type Output = CurveElGamalCiphertext;

    fn mul(self, rhs: &Scalar) -> Self::Output {
        CurveElGamalCiphertext {
            c1: self.c1 * rhs,
            c2: self.c2 * rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
    use curve25519_dalek::scalar::Scalar;
    use rand_core::OsRng;
    use scicrypt_traits::randomness::SecureRng;
    use scicrypt_traits::cryptosystems::AsymmetricCryptosystem;
    use scicrypt_traits::Enrichable;
    use crate::cryptosystems::curve_el_gamal::CurveElGamal;

    #[test]
    fn test_encrypt_decrypt_generator() {
        let mut rng = SecureRng::new(OsRng);

        let (pk, sk) = CurveElGamal::generate_keys(&Default::default(), &mut rng);

        let ciphertext = CurveElGamal::encrypt(&RISTRETTO_BASEPOINT_POINT, &pk, &mut rng);

        assert_eq!(
            RISTRETTO_BASEPOINT_POINT,
            CurveElGamal::decrypt(&ciphertext.enrich(&pk), &sk)
        );
    }

    #[test]
    fn test_probabilistic_encryption() {
        let mut rng = SecureRng::new(OsRng);

        let (pk, _) = CurveElGamal::generate_keys(&Default::default(), &mut rng);

        let ciphertext1 = CurveElGamal::encrypt(&RISTRETTO_BASEPOINT_POINT, &pk, &mut rng);
        let ciphertext2 = CurveElGamal::encrypt(&RISTRETTO_BASEPOINT_POINT, &pk, &mut rng);

        assert_ne!(ciphertext1, ciphertext2);
    }

    #[test]
    fn test_homomorphic_add() {
        let mut rng = SecureRng::new(OsRng);

        let (pk, sk) = CurveElGamal::generate_keys(&Default::default(), &mut rng);

        let ciphertext = CurveElGamal::encrypt(&RISTRETTO_BASEPOINT_POINT, &pk, &mut rng);
        let ciphertext_twice = &ciphertext + &ciphertext;

        assert_eq!(
            &Scalar::from(2u64) * &RISTRETTO_BASEPOINT_POINT,
            CurveElGamal::decrypt(&ciphertext_twice.enrich(&pk), &sk)
        );
    }

    #[test]
    fn test_homomorphic_scalar_mul() {
        let mut rng = SecureRng::new(OsRng);

        let (pk, sk) = CurveElGamal::generate_keys(&Default::default(), &mut rng);

        let ciphertext = CurveElGamal::encrypt(&RISTRETTO_BASEPOINT_POINT, &pk, &mut rng);
        let ciphertext_thrice = &ciphertext * &Scalar::from(3u64);

        assert_eq!(
            &Scalar::from(3u64) * &RISTRETTO_BASEPOINT_POINT,
            CurveElGamal::decrypt(&ciphertext_thrice.enrich(&pk), &sk)
        );
    }
}