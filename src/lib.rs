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

impl<T> Signed<T>
where
    for<'a> &'a T: DigestHash,
{
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

impl<T: DigestHash + Copy> Sign for T {
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
#[serde(try_from = "&[u8]")]
pub struct PubKey(identity::PublicKey);

impl From<PubKey> for Vec<u8> {
    fn from(value: PubKey) -> Self {
        value.0.encode_protobuf()
    }
}

impl TryFrom<&[u8]> for PubKey {
    type Error = identity::DecodingError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(PubKey(identity::PublicKey::try_decode_protobuf(value)?))
    }
}

pub trait Encode: Sized {
    type Error;
    fn encode(self, writer: impl io::Write) -> Result<(), Self::Error>;

    fn encode_to_vec(self) -> Result<Vec<u8>, Self::Error> {
        let mut buff = Vec::new();
        self.encode(&mut buff)?;
        Ok(buff)
    }
}

pub trait Decode: Sized {
    type Error;
    fn decode(reader: impl io::Read) -> Result<Self, Self::Error>;
}

impl<T: serde::Serialize> Encode for &T {
    type Error = bincode::Error;
    fn encode(self, writer: impl io::Write) -> Result<(), Self::Error> {
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
    fn digest_hash(self) -> Result<[u8; 32], Self::Error>;
}

impl<T: Encode> DigestHash for T {
    type Error = T::Error;

    fn digest_hash(self) -> Result<[u8; 32], Self::Error> {
        let mut hasher = sha3::Keccak256::new();

        hasher.update(&self.encode_to_vec()?);

        Ok(hasher.finalize().into())
    }
}

use std::io;

use libp2p::identity;
use sha3::Digest;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes_can_be_signed_and_verified() {
        todo!();
    }
}
