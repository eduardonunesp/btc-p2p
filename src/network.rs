use super::errors::{BTCP2PError, Result};

/// Represents the network to which a message belongs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    /// Mainnet
    /// Default Port 8333
    MainNet,

    /// Testnet
    /// Default Port 18333
    TestNet,

    /// Regtest
    /// Default Port 18444
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

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, TestResult};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for Network {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            match u8::arbitrary(g) % 3 {
                0 => Self::MainNet,
                1 => Self::TestNet,
                2 => Self::RegTest,
                _ => unreachable!(),
            }
        }
    }

    #[quickcheck]
    fn test_to_bytes(network: Network) -> TestResult {
        let bytes = Network::to_bytes(network);
        let network2 = Network::from_bytes(&bytes).unwrap();
        TestResult::from_bool(network == network2)
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
