use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use sha2::{Digest, Sha256};
use std::io::Write;
use Ok;

use super::errors::BTCP2PError;

use super::{
    command::Command,
    encode::{Decodable, Encodable},
    errors::Result,
    network::Network,
    payload::Payload,
};

const START_STRING_SIZE: usize = 4;
const COMMAND_NAME_SIZE: usize = 12;
const PAYLOAD_LEN_SIZE: usize = 4;
const CHECKSUM_SIZE: usize = 4;
const HEADER_SIZE: usize = START_STRING_SIZE + COMMAND_NAME_SIZE + PAYLOAD_LEN_SIZE + CHECKSUM_SIZE;

// 32 MB
const MAX_PAYLOAD_SIZE: usize = 32 * 1024 * 1024;

const HEADER_START_STRING_RANGE: std::ops::Range<usize> = 0..4;
const HEADER_COMMAND_NAME_RANGE: std::ops::Range<usize> = 4..16;
const HEADER_PAYLOAD_LEN_RANGE: std::ops::Range<usize> = 16..20;
const HEADER_CHECKSUM_RANGE: std::ops::Range<usize> = 20..24;

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

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        // buffer for the BTC proto: https://developer.bitcoin.org/reference/p2p_networking.html#message-headers
        let payload_bytes = self.payload.to_bytes()?;
        let mut buffer = Vec::with_capacity(HEADER_SIZE + payload_bytes.len());

        // start string char[4]
        buffer.write_all(&self.network.to_bytes()?)?;

        // command name char[12]
        let command_bytes = self.command.to_bytes()?;
        buffer.extend(&command_bytes);

        // padding char[..]
        (0..COMMAND_NAME_SIZE - command_bytes.len()).try_for_each(|_| buffer.write_u8(0x0))?;

        // payload length uint32 (4 bytes)
        buffer.write_u32::<LittleEndian>(payload_bytes.len() as u32)?;

        // checksum char[4]
        buffer.extend(&Message::checksum(&payload_bytes));

        // 24 bytes written so far

        // payload char[..] (variable length)
        buffer.extend(&payload_bytes);

        Ok(buffer)
    }

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

impl Encodable for Message {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Message::to_bytes(self)
    }
}

impl Decodable for Message {
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        Message::from_bytes(bytes)
    }
}
