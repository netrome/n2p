#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct Note {
    pub topic: String,
    pub msg: String,
    pub created_at: time::PrimitiveDateTime,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Signed<T> {
    pub inner: T,
    pub pub_key: PubKey,
    pub signature: Vec<u8>,
}

impl<T: DigestHash> Signed<T> {
    pub fn verify(&self) -> bool {
        let Ok(inner_digest_hash) = self.inner.digest_hash() else {
            return false;
        };

        self.pub_key.0.verify(&inner_digest_hash, &self.signature)
    }
}

pub trait Sign: Sized {
    type Error;
    fn sign(self, key_pair: &identity::Keypair) -> Result<Signed<Self>, Self::Error>;
}

impl<T: DigestHash> Sign for T {
    type Error = SignError<T::Error>;
    fn sign(self, key_pair: &identity::Keypair) -> Result<Signed<Self>, Self::Error> {
        let pub_key = PubKey(key_pair.public());
        let signature = key_pair.sign(&self.digest_hash().map_err(SignError::DigestError)?)?;

        Ok(Signed {
            inner: self,
            pub_key,
            signature,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SignError<DigestError> {
    #[error("signing error")]
    SigningError(#[from] identity::SigningError),
    #[error("digest error")]
    DigestError(DigestError),
}
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(into = "Vec<u8>")]
#[serde(try_from = "Vec<u8>")]
pub struct PubKey(identity::PublicKey);

impl From<PubKey> for Vec<u8> {
    fn from(value: PubKey) -> Self {
        value.0.encode_protobuf()
    }
}

impl TryFrom<Vec<u8>> for PubKey {
    type Error = identity::DecodingError;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(PubKey(identity::PublicKey::try_decode_protobuf(&value)?))
    }
}

pub trait Encode: Sized {
    type Error;
    fn encode(&self, writer: impl io::Write) -> Result<(), Self::Error>;

    fn encode_to_vec(&self) -> Result<Vec<u8>, Self::Error> {
        let mut buff = Vec::new();
        self.encode(&mut buff)?;
        Ok(buff)
    }
}

pub trait Decode: Sized {
    type Error;
    fn decode(reader: impl io::Read) -> Result<Self, Self::Error>;
}

impl<T: serde::Serialize> Encode for T {
    type Error = bincode::Error;
    fn encode(&self, writer: impl io::Write) -> Result<(), Self::Error> {
        bincode::serialize_into(writer, self)
    }
}

impl<T: serde::de::DeserializeOwned> Decode for T {
    type Error = bincode::Error;
    fn decode(reader: impl io::Read) -> Result<Self, Self::Error> {
        bincode::deserialize_from(reader)
    }
}

pub trait DigestHash {
    type Error;
    fn digest_hash(&self) -> Result<[u8; 32], Self::Error>;
}

impl<T: Encode> DigestHash for T {
    type Error = T::Error;

    fn digest_hash(&self) -> Result<[u8; 32], Self::Error> {
        let mut hasher = sha3::Keccak256::new();

        hasher.update(self.encode_to_vec()?);

        Ok(hasher.finalize().into())
    }
}

use std::io;

use libp2p::identity;
use sha3::Digest;

#[cfg(test)]
mod tests {
    use super::*;

    use fake::Fake;
    use rand::{RngCore, SeedableRng};

    #[test]
    fn notes_can_be_signed_and_verified() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut secret_key_bytes = [0; 32];
        rng.fill_bytes(&mut secret_key_bytes);
        let keypair = identity::Keypair::ed25519_from_bytes(secret_key_bytes)
            .expect("Failed to generate keypair");

        let note: Note = fake::Faker.fake_with_rng(&mut rng);

        let mut signed = note.sign(&keypair).expect("Failed to sign note");

        assert!(signed.verify());

        signed.inner.msg.push_str("TAMPERED");

        assert!(!signed.verify());
    }

    #[test]
    fn notes_can_be_encoded_and_decoded() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut secret_key_bytes = [0; 32];
        rng.fill_bytes(&mut secret_key_bytes);
        let keypair = identity::Keypair::ed25519_from_bytes(secret_key_bytes)
            .expect("failed to generate keypair");

        let note: Note = fake::Faker.fake_with_rng(&mut rng);

        let signed = note.sign(&keypair).expect("failed to sign note");

        let encoded = signed
            .encode_to_vec()
            .expect("failed to encode signed note");
        let decoded =
            Signed::<Note>::decode(encoded.as_slice()).expect("failed to decoded encoded note");

        assert_eq!(decoded, signed);
    }
}
