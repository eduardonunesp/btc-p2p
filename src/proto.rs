use std::io::{Read, Write};

use anyhow::{bail, Ok};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use sha2::{Digest, Sha256};

const START_STRING: [u8; 4] = [0xf9, 0xbe, 0xb4, 0xd9];
const COMMAND_NAME_SIZE: usize = 12;
const PAYLOAD_LEN_SIZE: usize = 4;
const CHECKSUM_SIZE: usize = 4;
const HEADER_SIZE: usize =
    START_STRING.len() + COMMAND_NAME_SIZE + PAYLOAD_LEN_SIZE + CHECKSUM_SIZE;

// 32 MB
const MAX_PAYLOAD_SIZE: usize = 32 * 1024 * 1024;

const HEADER_START_STRING_RANGE: std::ops::Range<usize> = 0..4;
const HEADER_COMMAND_NAME_RANGE: std::ops::Range<usize> = 4..16;
const HEADER_PAYLOAD_LEN_RANGE: std::ops::Range<usize> = 16..20;
const HEADER_CHECKSUM_RANGE: std::ops::Range<usize> = 20..24;

#[derive(Debug)]
pub enum Command {
    Pong,
    Ping,
    Version,
    VerAck,
}

impl Command {
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        match self {
            Command::VerAck => Ok("verack".as_bytes().into()),
            Command::Pong => Ok("".as_bytes().into()),
            Command::Ping => Ok("ping".as_bytes().into()),
            Command::Version => Ok("version".as_bytes().into()),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let command = String::from_utf8(bytes.to_vec())?.replace('\0', "");

        Ok(match command.as_str() {
            "verack" => Self::VerAck,
            "ping" => Self::Ping,
            "version" => Self::Version,
            c => {
                tracing::info!("Command::from_bytes: {:?}", c);
                Self::Version
            }
        })
    }
}

#[derive(Debug)]
pub enum Payload {
    Empty,
    VerAck,
    Ping(u64),
    Version(VersionPayload),
}

impl Payload {
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        match self {
            Payload::Empty => bail!("Empty payload not supported"),
            Payload::VerAck => Ok(vec![]),
            Payload::Ping(nonce) => {
                let mut buf: Vec<u8> = vec![];
                buf.write_u64::<LittleEndian>(nonce.to_owned())?;
                Ok(buf)
            }
            Payload::Version(payload) => {
                let mut buf: Vec<u8> = vec![];
                buf.write_i32::<LittleEndian>(payload.version)?;
                buf.write_u64::<LittleEndian>(payload.services)?;
                buf.write_i64::<LittleEndian>(payload.timestamp)?;
                buf.write_u64::<LittleEndian>(payload.addr_recv_serv)?;
                buf.write_u128::<BigEndian>(u128::from_ne_bytes(payload.addr_recv))?;
                buf.write_u16::<LittleEndian>(payload.addr_recv_port)?;
                buf.write_u64::<LittleEndian>(payload.addr_trans_serv)?;
                buf.write_u128::<BigEndian>(u128::from_ne_bytes(payload.addr_trans))?;
                buf.write_u16::<LittleEndian>(payload.addr_trans_port)?;
                buf.write_u64::<LittleEndian>(payload.nonce)?;
                buf.write_u8(payload.user_agent.len() as u8)?;
                buf.write_all(payload.user_agent.as_bytes())?;
                buf.write_i32::<LittleEndian>(payload.start_height)?;
                buf.write_u8(payload.relay.into())?;
                Ok(buf)
            }
        }
    }

    pub fn from_bytes(command: &Command, bytes: &[u8]) -> anyhow::Result<Self> {
        match command {
            Command::VerAck => Ok(Self::Empty),
            Command::Pong => Ok(Self::Empty),
            Command::Ping => {
                let mut buffer = bytes;
                let nonce = buffer.read_u64::<LittleEndian>()?;
                Ok(Self::Ping(nonce))
            }
            Command::Version => {
                let mut buffer = bytes;
                let version_payload = VersionPayload {
                    version: buffer.read_i32::<LittleEndian>()?,
                    services: buffer.read_u64::<LittleEndian>()?,
                    timestamp: buffer.read_i64::<LittleEndian>()?,
                    addr_recv_serv: buffer.read_u64::<LittleEndian>()?,
                    addr_recv: buffer.read_u128::<BigEndian>()?.to_ne_bytes(),
                    addr_recv_port: buffer.read_u16::<BigEndian>()?,
                    addr_trans_serv: buffer.read_u64::<LittleEndian>()?,
                    addr_trans: buffer.read_u128::<BigEndian>()?.to_ne_bytes(),
                    addr_trans_port: buffer.read_u16::<BigEndian>()?,
                    nonce: buffer.read_u64::<LittleEndian>()?,
                    user_agent: {
                        let mut tmp_buffer = vec![0u8; 0];
                        let user_agent_len = buffer.read_u8()?;
                        let user_agent_bytes = vec![0u8; user_agent_len as usize];
                        buffer.read_exact(&mut tmp_buffer)?;
                        String::from_utf8(user_agent_bytes)?
                    },
                    start_height: buffer.read_i32::<LittleEndian>()?,
                    relay: buffer.read_u8()? != 0x00,
                };

                Ok(Self::Version(version_payload))
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct VersionPayload {
    pub version: i32,
    pub services: u64,
    pub timestamp: i64,
    pub addr_recv_serv: u64,
    pub addr_recv: [u8; 16],
    pub addr_recv_port: u16,
    pub addr_trans_serv: u64,
    pub addr_trans: [u8; 16],
    pub addr_trans_port: u16,
    pub nonce: u64,
    pub user_agent: String,
    pub start_height: i32,
    pub relay: bool,
}

#[derive(Debug)]
pub struct Message {
    pub start_string: [u8; 4],
    pub command: Command,
    pub payload: Payload,
}

impl Message {
    pub fn new(payload: Payload) -> Self {
        let start_string = START_STRING;
        let command = match payload {
            Payload::Empty => Command::Pong,
            Payload::VerAck => Command::VerAck,
            Payload::Ping(_) => Command::Ping,
            Payload::Version(_) => Command::Version,
        };

        Self {
            start_string,
            command,
            payload,
        }
    }

    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        // buffer for the BTC proto: https://developer.bitcoin.org/reference/p2p_networking.html#message-headers
        let mut buffer = vec![];

        // start string char[4]
        buffer.write_all(&self.start_string)?;

        // command name char[12]
        buffer.extend_from_slice(&self.command.to_bytes()?.as_slice());
        let command_len = self.command.to_bytes()?.len();
        (0..COMMAND_NAME_SIZE - command_len).try_for_each(|_| buffer.write_u8(0x00))?;

        // payload length uint32
        buffer.write_u32::<LittleEndian>(self.payload.to_bytes()?.len() as u32)?;

        // checksum char[4]
        buffer.extend_from_slice(&checksum(&self.payload.to_bytes()?));

        // payload char[..]
        buffer.extend_from_slice(&self.payload.to_bytes()?.as_slice());
        Ok(buffer)
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() < HEADER_SIZE {
            anyhow::bail!("Invalid header size");
        }

        // start string char[4]
        let mut start_string = [0u8; 4];

        if &bytes[HEADER_START_STRING_RANGE] != &START_STRING {
            anyhow::bail!("Invalid start string");
        }

        start_string.copy_from_slice(&bytes[HEADER_START_STRING_RANGE]);

        // command name char[12]
        let command = Command::from_bytes(&bytes[HEADER_COMMAND_NAME_RANGE])?;

        // payload length uint32 (4 bytes)
        let payload_len = (&bytes[HEADER_PAYLOAD_LEN_RANGE]).read_u32::<LittleEndian>()?;
        if payload_len > MAX_PAYLOAD_SIZE as u32 {
            anyhow::bail!("Payload too large");
        }

        // checksum char[4]
        let checksum_value = &bytes[HEADER_CHECKSUM_RANGE];

        // payload char[..]
        let payload_data = &bytes[HEADER_CHECKSUM_RANGE.end..];

        if checksum_value != checksum(&payload_data) {
            anyhow::bail!("Checksum mismatch");
        }

        let payload = Payload::from_bytes(&command, payload_data)?;

        Ok(Self {
            start_string,
            command,
            payload,
        })
    }
}

pub fn checksum(data: &[u8]) -> [u8; 4] {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_to_bytes() {
        let command = Command::Ping;
        let bytes = command.to_bytes().unwrap();

        assert_eq!(bytes, vec![112, 105, 110, 103]);
    }

    #[test]
    fn fn_test_payload_to_bytes() {
        let payload = Payload::Ping(256);
        let bytes = payload.to_bytes().unwrap();

        assert_eq!(bytes, vec![0, 1, 0, 0, 0, 0, 0, 0]);
    }
}
