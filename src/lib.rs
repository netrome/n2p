#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Note {
    topic: String,
    msg: String,
    created_at: time::PrimitiveDateTime,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Signed<T> {
    inner: T,
    pub_key: PubKey,
    signature: Vec<u8>,
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

use libp2p::identity;
