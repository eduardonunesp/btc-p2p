use super::errors::Result;

pub trait Encodable {
    fn to_bytes(&self) -> Result<Vec<u8>>;
}

pub trait Decodable {
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

pub fn encode<T: Encodable>(object: &T) -> Result<Vec<u8>> {
    object.to_bytes()
}

pub fn decode<T: Decodable>(bytes: &[u8]) -> Result<T> {
    T::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Test {
        value: u32,
    }

    impl Encodable for Test {
        fn to_bytes(&self) -> Result<Vec<u8>> {
            Ok(self.value.to_be_bytes().to_vec())
        }
    }

    impl Decodable for Test {
        fn from_bytes(bytes: &[u8]) -> Result<Self> {
            let value = u32::from_be_bytes(bytes.try_into()?);

            Ok(Self { value })
        }
    }

    #[test]
    fn test_encode_decode() {
        let test = Test { value: 256 };
        let bytes = encode(&test).unwrap();
        let decoded = decode::<Test>(&bytes).unwrap();

        assert_eq!(decoded.value, test.value);
    }
}
