use super::{
    encode::{Decodable, Encodable},
    errors::Result,
};

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
            c => Self::Version,
        })
    }
}

impl Encodable for Command {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Command::to_bytes(self)
    }
}

impl Decodable for Command {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Command::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_bytes() {
        assert_eq!(
            Command::to_bytes(&Command::Version).unwrap(),
            "version".as_bytes(),
        );
        assert_eq!(
            Command::to_bytes(&Command::VerAck).unwrap(),
            "verack".as_bytes(),
        );
        assert_eq!(
            Command::to_bytes(&Command::Ping).unwrap(),
            "ping".as_bytes(),
        );
        assert_eq!(
            Command::to_bytes(&Command::Pong).unwrap(),
            "pong".as_bytes(),
        );
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
