use super::errors::{BTCP2PError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Version,
    VerAck,
    Ping,
    Pong,
}

impl Command {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Command::Version => Ok("version".into()),
            Command::VerAck => Ok("verack".into()),
            Command::Ping => Ok("ping".into()),
            Command::Pong => Ok("pong".into()),
        }
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
