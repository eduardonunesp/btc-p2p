use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use sha2::{Digest, Sha256};
use std::io::Write;

use super::{
    command::Command,
    errors::{BTCP2PError, Result},
    network::Network,
    payload::Payload,
    CHECKSUM_SIZE, HEADER_CHECKSUM_RANGE, HEADER_COMMAND_NAME_RANGE, HEADER_PAYLOAD_LEN_RANGE,
    HEADER_SIZE, HEADER_START_STRING_RANGE, MAX_PAYLOAD_SIZE,
};

/// Message represents a message in the BTC proto
/// Contains the network, command and payload
///
/// All messages in the network protocol use the same container format, which provides a required multi-field message header and an optional payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub network: Network,
    pub command: Command,
    pub payload: Payload,
}

impl Message {
    pub fn new(network: Network, command: Command, payload: Payload) -> Self {
        Self {
            network,
            command,
            payload,
        }
    }

    /// Converts the message to bytes
    /// Bytes are contained in a Vec<u8>
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        // buffer for the BTC proto: https://developer.bitcoin.org/reference/p2p_networking.html#message-headers
        let payload_bytes = self.payload.to_bytes()?;
        let mut buffer = Vec::with_capacity(HEADER_SIZE + payload_bytes.len());

        // start string char[4]
        buffer.write_all(Network::to_bytes(self.network).as_slice())?;

        // command name char[12]
        let command_bytes = self.command.to_bytes()?;
        buffer.extend(&command_bytes);

        // payload length uint32 (4 bytes)
        buffer.write_u32::<LittleEndian>(payload_bytes.len() as u32)?;

        // checksum char[4]
        buffer.extend(&Message::checksum(&payload_bytes));

        // 24 bytes written so far

        // payload char[..] (variable length)
        buffer.extend(&payload_bytes);

        Ok(buffer)
    }

    /// Converts bytes to a message
    /// Bytes are contained in a slice of u8
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_SIZE {
            return Err(BTCP2PError::InvalidHeaderSize);
        }

        // start string char[4]
        let network = Network::from_bytes(&bytes[HEADER_START_STRING_RANGE])?;

        // command name char[12]
        let command = Command::from_bytes(&bytes[HEADER_COMMAND_NAME_RANGE])?;

        // payload length uint32 (4 bytes)
        let payload_len = (&bytes[HEADER_PAYLOAD_LEN_RANGE]).read_u32::<LittleEndian>()?;
        if payload_len > MAX_PAYLOAD_SIZE as u32 {
            return Err(BTCP2PError::PayloadTooLarge);
        }

        // checksum char[4]
        let checksum_value = &bytes[HEADER_CHECKSUM_RANGE];

        // payload char[..]
        let payload_bytes = &bytes[HEADER_CHECKSUM_RANGE.end..];

        if checksum_value != Message::checksum(&payload_bytes) {
            return Err(BTCP2PError::InvalidChecksum);
        }

        let payload = Payload::from_bytes(&command, payload_bytes)?;

        Ok(Self {
            network,
            command,
            payload,
        })
    }

    /// Calculates the checksum of the payload
    fn checksum(data: &[u8]) -> [u8; 4] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(hash);
        let hash = hasher.finalize();

        let mut buffer = [0u8; CHECKSUM_SIZE];
        buffer.clone_from_slice(&hash[..CHECKSUM_SIZE]);

        buffer
    }
}

#[cfg(test)]
mod tests {
    use crate::VersionPayload;

    use super::*;
    use quickcheck::{Arbitrary, TestResult};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for Message {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let network = Network::arbitrary(g);
            let command = Command::arbitrary(g);
            let payload = match command {
                Command::Version => Payload::Version(VersionPayload::arbitrary(g)),
                Command::VerAck => Payload::VerAck,
                Command::Ping => Payload::Ping(u64::arbitrary(g)),
                Command::Pong => Payload::Pong(u64::arbitrary(g)),
            };

            Self {
                network,
                command,
                payload,
            }
        }
    }

    #[quickcheck]
    fn test_to_bytes(message: Message) -> TestResult {
        let bytes = message.to_bytes().unwrap();
        let message2 = Message::from_bytes(&bytes).unwrap();
        TestResult::from_bool(message == message2)
    }
}
