use ed25519_compact::{KeyPair, PublicKey, SecretKey, Seed, Signature};
use rand_core::{CryptoRng, RngCore};

use std::borrow::Cow;

use crate::{
    alg::{SigningKey, VerifyingKey},
    Algorithm, AlgorithmSignature, Renamed,
};

impl AlgorithmSignature for Signature {
    fn try_from_slice(bytes: &[u8]) -> anyhow::Result<Self> {
        let mut signature = [0u8; Signature::BYTES];
        if bytes.len() != signature.len() {
            return Err(ed25519_compact::Error::SignatureMismatch.into());
        }
        signature.copy_from_slice(bytes);
        Ok(Self::new(signature))
    }

    fn as_bytes(&self) -> Cow<[u8]> {
        Cow::Borrowed(self.as_ref())
    }
}

/// Integrity algorithm using digital signatures on the Ed25519 elliptic curve.
///
/// The name of the algorithm is specified as `EdDSA` as per the [IANA registry].
/// Use `with_specific_name()` to switch to non-standard `Ed25519`.
///
/// *This type is available if the crate is built with the `ed25519-compact` feature.*
///
/// [IANA registry]: https://www.iana.org/assignments/jose/jose.xhtml
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ed25519;

impl Ed25519 {
    /// Creates an algorithm instance with the algorithm name specified as `Ed25519`.
    /// This is a non-standard name, but it is used in some apps.
    pub fn with_specific_name() -> Renamed<Self> {
        Renamed::new(Self, "Ed25519")
    }

    /// Generate a new key pair.
    pub fn generate<R: CryptoRng + RngCore>(&self, rng: &mut R) -> (SecretKey, PublicKey) {
        let mut seed = [0u8; Seed::BYTES];
        rng.fill_bytes(&mut seed);
        let keypair = KeyPair::from_seed(Seed::new(seed));
        (keypair.sk, keypair.pk)
    }
}

impl Algorithm for Ed25519 {
    type SigningKey = SecretKey;
    type VerifyingKey = PublicKey;
    type Signature = Signature;

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("EdDSA")
    }

    fn sign(&self, signing_key: &Self::SigningKey, message: &[u8]) -> Self::Signature {
        signing_key.sign(message, Some(Default::default()))
    }

    fn verify_signature(
        &self,
        signature: &Self::Signature,
        verifying_key: &Self::VerifyingKey,
        message: &[u8],
    ) -> bool {
        verifying_key.verify(message, signature).is_ok()
    }
}

impl VerifyingKey<Ed25519> for PublicKey {
    fn from_slice(raw: &[u8]) -> anyhow::Result<Self> {
        Self::from_slice(raw).map_err(From::from)
    }

    fn as_bytes(&self) -> Cow<[u8]> {
        Cow::Borrowed(self.as_ref())
    }
}

impl SigningKey<Ed25519> for SecretKey {
    fn from_slice(raw: &[u8]) -> anyhow::Result<Self> {
        Self::from_slice(raw).map_err(From::from)
    }

    fn to_verifying_key(&self) -> PublicKey {
        self.public_key()
    }

    fn as_bytes(&self) -> Cow<[u8]> {
        Cow::Borrowed(self.as_ref())
    }
}