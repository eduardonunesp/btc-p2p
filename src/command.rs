use super::COMMAND_NAME_SIZE;

use super::errors::{BTCP2PError, Result};

/// Command represents a command in the BTC proto
/// https://developer.bitcoin.org/reference/p2p_networking.html#message-headers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Version,
    VerAck,
    Ping,
    Pong,
}

impl Command {
    /// to_bytes converts the command to bytes
    /// the bytes are the command name
    /// bytes are padded with null bytes according to the protocol
    /// https://developer.bitcoin.org/reference/p2p_networking.html#message-headers
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut result = match self {
            Command::Version => "version".to_string(),
            Command::VerAck => "verack".to_string(),
            Command::Ping => "ping".to_string(),
            Command::Pong => "pong".to_string(),
        };

        // padding with null bytes
        for _ in 0..COMMAND_NAME_SIZE - result.len() {
            result.push('\0');
        }

        Ok(result.as_bytes().to_vec())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let command = String::from_utf8(bytes.to_vec())?.replace('\0', "");

        Ok(match command.as_str() {
            "version" => Self::Version,
            "verack" => Self::VerAck,
            "ping" => Self::Ping,
            "pong" => Self::Pong,
            _ => return Err(BTCP2PError::InvalidCommand),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, TestResult};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for Command {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            match u8::arbitrary(g) % 4 {
                0 => Self::Version,
                1 => Self::VerAck,
                2 => Self::Ping,
                3 => Self::Pong,
                _ => unreachable!(),
            }
        }
    }

    #[quickcheck]
    fn test_to_bytes(command: Command) -> TestResult {
        let bytes = command.to_bytes().unwrap();
        let command2 = Command::from_bytes(&bytes).unwrap();
        TestResult::from_bool(command == command2)
    }

    #[test]
    fn test_from_bytes() {
        assert_eq!(
            Command::from_bytes("version".as_bytes()).unwrap(),
            Command::Version
        );
        assert_eq!(
            Command::from_bytes("verack".as_bytes()).unwrap(),
            Command::VerAck
        );
        assert_eq!(
            Command::from_bytes("ping".as_bytes()).unwrap(),
            Command::Ping
        );
        assert_eq!(
            Command::from_bytes("pong".as_bytes()).unwrap(),
            Command::Pong
        );
        assert_eq!(
            Command::from_bytes("version\0\0\0\0".as_bytes()).unwrap(),
            Command::Version
        );
    }
}
