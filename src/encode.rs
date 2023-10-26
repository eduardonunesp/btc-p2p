use super::errors::Result;

pub trait Encodable {
    fn to_bytes(&self) -> Result<Vec<u8>>;
}

pub trait Decodable {
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
}
