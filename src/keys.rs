use crate::{
    constants::{
        SECRETKEY_SEED_LENGTH,
        SECRETKEY_SCALAR_LENGTH,
        SECRETKEY_NONCE_LENGTH,

        // PUBLICKEY_LENGTH,

        SHA512_LENGTH,
    },
    curve::{
        CurvePoint,
        CompressedY,
    },
    hash::Hash as Sha512,
    scalar::{
        Scalar,
        TweetNaclScalar,
    },
};

pub struct SecretKey {
    #[allow(dead_code)]
    pub (crate) seed: [u8; SECRETKEY_SEED_LENGTH],
    pub (crate) scalar: Scalar,
    pub (crate) nonce: [u8; SECRETKEY_NONCE_LENGTH],
}

pub struct PublicKey {
    #[allow(dead_code)]
    pub(crate) point: CurvePoint,
    pub(crate) compressed: CompressedY,
}

pub struct Keypair {
    pub secret: SecretKey,
    pub public: PublicKey,
}

pub struct Signature {
    pub r: CompressedY,
    pub s: Scalar,
}

impl Keypair {
    pub fn sign(&self, message: &[u8]) -> Signature {

        // R = rB, with r = H(nonce, M)
        let first_hash = Sha512::new()
            .updated(&self.secret.nonce)
            .updated(message)
            .finalize();

        let r: Scalar = Scalar::from_u512_le(&first_hash);
        #[allow(non_snake_case)]
        let R: CompressedY = (&r * &CurvePoint::basepoint()).compressed();


        // S = r + H(R, A, M)s (mod l), with A = sB the public key
        let second_hash = Sha512::new()
            .updated(&R.0)
            .updated(&self.public.compressed.0)
            .updated(message)
            .finalize();

        let h: Scalar = Scalar::from_u512_le(&second_hash);
        let mut s = &r.into() + &(&h.into() * &TweetNaclScalar::from(&self.secret.scalar));
        let s = s.reduce_modulo_ell();

        Signature { r: R, s }
    }

    pub fn sign_prehashed(&self, prehashed_message: &[u8; SHA512_LENGTH], context: Option<&'static [u8]>)
    -> Signature {
        // By default, the context is an empty string.
        let context: &[u8] = context.unwrap_or(b"");
        debug_assert!(context.len() <= 255, "The context must not be longer than 255 octets.");

        let first_hash = Sha512::new()
            // Ed25519ph parts
            .updated(b"SigEd25519 no Ed25519 collisions")
            .updated(&[1])
            // context parts
            .updated(&[context.len() as u8])
            .updated(context)
            // usual parts
            .updated(&self.secret.nonce)
            .updated(prehashed_message)
            .finalize();

        // from here on, same as normal signing
        let r: Scalar = Scalar::from_u512_le(&first_hash);
        #[allow(non_snake_case)]
        let R: CompressedY = (&r * &CurvePoint::basepoint()).compressed();


        let second_hash = Sha512::new()
            // Ed25519ph parts
            .updated(b"SigEd25519 no Ed25519 collisions")
            .updated(&[1])
            // context parts
            .updated(&[context.len() as u8])
            .updated(context)
            // usual parts
            .updated(&R.0)
            .updated(&self.public.compressed.0)
            .updated(prehashed_message)
            .finalize();

        let h: Scalar = Scalar::from_u512_le(&second_hash);
        let mut s = &r.into() + &(&h.into() * &TweetNaclScalar::from(&self.secret.scalar));
        let s = s.reduce_modulo_ell();

        Signature { r: R, s }
    }
}

impl From<&[u8; SECRETKEY_SEED_LENGTH]> for SecretKey {
    fn from(seed: &[u8; SECRETKEY_SEED_LENGTH]) -> SecretKey {

        let mut hash: Sha512 = Sha512::new();
        hash.update(seed);
        let digest = hash.finalize();

        let mut scalar_bytes = [0u8; 32];
        scalar_bytes.copy_from_slice(&digest[..SECRETKEY_SCALAR_LENGTH]);
        let mut scalar = Scalar(scalar_bytes);
        // let mut scalar = Scalar::from_bytes(&digest[..SECRETKEY_SCALAR_LENGTH]);
        scalar.0[0] &= 248;
        scalar.0[31] &= 127;
        scalar.0[31] |= 64;

        let mut nonce = [0u8; SECRETKEY_NONCE_LENGTH];
        nonce.copy_from_slice(&digest[SECRETKEY_SCALAR_LENGTH..]);

        SecretKey { seed: seed.clone(), scalar, nonce }
    }
}

impl From<&SecretKey> for PublicKey {
    fn from(secret: &SecretKey) -> PublicKey {

        let point = &secret.scalar * &CurvePoint::basepoint();
        let compressed = point.compressed();

        PublicKey { point, compressed }
    }
}

impl From<&[u8; SECRETKEY_SEED_LENGTH]> for Keypair {
    fn from(seed: &[u8; SECRETKEY_SEED_LENGTH]) -> Keypair {
        let secret = SecretKey::from(seed);

        let public = PublicKey::from(&secret);

        Keypair { secret, public }
    }
}

// TODO: to_bytes and from_bytes methods for secretkey, publickey and keypair

#[cfg(test)]
mod tests {

    use super::Keypair;
    use crate::hash::Hash as Sha512;

    #[test]
    fn test_signature() {

        #![allow(non_snake_case)]

        let seed: [u8; 32] = [
            0x35, 0xb3, 0x07, 0x76, 0x17, 0x9a, 0x78, 0x58,
            0x34, 0xf0, 0x4c, 0x82, 0x88, 0x59, 0x5d, 0xf4,
            0xac, 0xa1, 0x0b, 0x33, 0xaa, 0x12, 0x10, 0xad,
            0xec, 0x3e, 0x82, 0x47, 0x25, 0x3e, 0x6c, 0x65,
        ];

        let keypair = Keypair::from(&seed);

        let data = "salty!".as_bytes();

        let R_expected = [
            0xec, 0x97, 0x27, 0x40, 0x07, 0xe7, 0x08, 0xc6,
            0xd1, 0xee, 0xd6, 0x01, 0x9f, 0x5d, 0x0f, 0xcb,
            0xe1, 0x8a, 0x67, 0x70, 0x8d, 0x17, 0x92, 0x4b,
            0x95, 0xdb, 0x7e, 0x35, 0xcc, 0xaa, 0x06, 0x3a,
        ];

        let S_expected = [
            0xb8, 0x64, 0x8c, 0x9b, 0xf5, 0x48, 0xb0, 0x09,
            0x90, 0x6f, 0xa1, 0x31, 0x09, 0x0f, 0xfe, 0x85,
            0xa1, 0x7e, 0x89, 0x99, 0xb8, 0xc4, 0x2c, 0x97,
            0x32, 0xf9, 0xa6, 0x44, 0x2a, 0x17, 0xbc, 0x09,
        ];

        let signature = keypair.sign(&data);

        assert_eq!(signature.r.0, R_expected);
        assert_eq!(signature.s.0, S_expected);
    }

    #[test]
    fn test_ed25519ph_with_rf8032_test_vector() {
        let seed: [u8; 32] = [
            0x83, 0x3f, 0xe6, 0x24, 0x09, 0x23, 0x7b, 0x9d,
            0x62, 0xec, 0x77, 0x58, 0x75, 0x20, 0x91, 0x1e,
            0x9a, 0x75, 0x9c, 0xec, 0x1d, 0x19, 0x75, 0x5b,
            0x7d, 0xa9, 0x01, 0xb9, 0x6d, 0xca, 0x3d, 0x42,
        ];

        let keypair = Keypair::from(&seed);

        let message: [u8; 3] = [0x61, 0x62, 0x63];

        let prehashed_message = Sha512::new().updated(&message).finalize();

        let signature = keypair.sign_prehashed(&prehashed_message, None);

        let expected_r = [
            0x98, 0xa7, 0x02, 0x22, 0xf0, 0xb8, 0x12, 0x1a,
            0xa9, 0xd3, 0x0f, 0x81, 0x3d, 0x68, 0x3f, 0x80,
            0x9e, 0x46, 0x2b, 0x46, 0x9c, 0x7f, 0xf8, 0x76,
            0x39, 0x49, 0x9b, 0xb9, 0x4e, 0x6d, 0xae, 0x41,
        ];

        let expected_s = [
            0x31, 0xf8, 0x50, 0x42, 0x46, 0x3c, 0x2a, 0x35,
            0x5a, 0x20, 0x03, 0xd0, 0x62, 0xad, 0xf5, 0xaa,
            0xa1, 0x0b, 0x8c, 0x61, 0xe6, 0x36, 0x06, 0x2a,
            0xaa, 0xd1, 0x1c, 0x2a, 0x26, 0x08, 0x34, 0x06,
        ];

        assert_eq!(signature.r.0, expected_r);
        assert_eq!(signature.s.0, expected_s);
    }
}
