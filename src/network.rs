use super::{
    encode::{Decodable, Encodable},
    errors::{BTCP2PError, Result},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    MainNet,
    TestNet,
    RegTest,
}

impl Network {
    pub fn to_bytes(network: Network) -> [u8; 4] {
        match network {
            Network::MainNet => [0xf9, 0xbe, 0xb4, 0xd9],
            Network::TestNet => [0x0b, 0x11, 0x09, 0x07],
            Network::RegTest => [0xfa, 0xbf, 0xb5, 0xda],
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        match bytes {
            [0xf9, 0xbe, 0xb4, 0xd9] => Ok(Self::MainNet),
            [0x0b, 0x11, 0x09, 0x07] => Ok(Self::TestNet),
            [0xfa, 0xbf, 0xb5, 0xda] => Ok(Self::RegTest),
            _ => Err(BTCP2PError::UnknowNetwork),
        }
    }
}

impl Encodable for Network {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(Network::to_bytes(*self).to_vec())
    }
}

impl Decodable for Network {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Network::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_bytes() {
        assert_eq!(
            Network::to_bytes(Network::MainNet),
            [0xf9, 0xbe, 0xb4, 0xd9]
        );
        assert_eq!(
            Network::to_bytes(Network::TestNet),
            [0x0b, 0x11, 0x09, 0x07]
        );
        assert_eq!(
            Network::to_bytes(Network::RegTest),
            [0xfa, 0xbf, 0xb5, 0xda]
        );
    }

    #[test]
    fn test_from_bytes() {
        assert_eq!(
            Network::from_bytes(&[0xf9, 0xbe, 0xb4, 0xd9]).unwrap(),
            Network::MainNet
        );
        assert_eq!(
            Network::from_bytes(&[0x0b, 0x11, 0x09, 0x07]).unwrap(),
            Network::TestNet
        );
        assert_eq!(
            Network::from_bytes(&[0xfa, 0xbf, 0xb5, 0xda]).unwrap(),
            Network::RegTest
        );
        assert_eq!(
            Network::from_bytes(&[0xde, 0xad, 0xbe, 0xef]).is_err(),
            true,
        );
    }
}
