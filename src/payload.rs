use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{Read, Write},
    net::SocketAddr,
    time::SystemTime,
};

use super::{command::Command, errors::Result};

const PROTOCOL_VERSION: i32 = 70015;

#[derive(Debug)]
pub enum Payload {
    Version(VersionPayload),
    VerAck,
    Ping(u64),
    Pong(u64),
    Empty,
}

impl Payload {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Payload::Version(version_payload) => Ok(version_payload.to_bytes()?.to_vec()),
            Payload::VerAck => Ok(vec![]),
            Payload::Ping(nonce) => Ok(nonce.to_le_bytes().to_vec()),
            Payload::Pong(nonce) => Ok(nonce.to_le_bytes().to_vec()),
            Payload::Empty => Ok(vec![]),
        }
    }

    pub fn from_bytes(command: &Command, bytes: &[u8]) -> Result<Self> {
        match command {
            Command::Version => Ok(Payload::Version(VersionPayload::from_bytes(&bytes)?)),
            Command::VerAck => Ok(Payload::VerAck),
            Command::Ping => Ok(Payload::Ping(u64::from_le_bytes(bytes.try_into()?))),
            Command::Pong => Ok(Payload::Empty),
        }
    }
}

pub struct ServiceFlags(u64);

impl ServiceFlags {
    /// This node is not a full node. It may not be able to provide any data except for the transactions it originates.
    pub const UNNAMED: ServiceFlags = ServiceFlags(0);

    /// This is a full node and can be asked for full blocks. It should implement all protocol features available in its self-reported protocol version
    pub const NODE_NETWORK: ServiceFlags = ServiceFlags(0x1);

    /// This is a full node capable of responding to the getutxo protocol request. This is not supported by any currently-maintained Bitcoin node.
    pub const NODE_GETUTXO: ServiceFlags = ServiceFlags(0x2);

    /// This is a full node capable and willing to handle bloom-filtered connections.
    pub const NODE_BLOOM: ServiceFlags = ServiceFlags(0x4);

    /// This is a full node that can be asked for blocks and transactions including witness data.
    pub const NODE_WITNESS: ServiceFlags = ServiceFlags(0x8);

    /// This is a full node that supports Xtreme Thinblocks. This is not supported by any currently-maintained Bitcoin node.
    pub const NODE_XTHIN: ServiceFlags = ServiceFlags(0x10);

    /// This is the same as NODE_NETWORK but the node has at least the last 288 blocks (last 2 days).
    pub const NODE_NETWORK_LIMITED: ServiceFlags = ServiceFlags(0x0400);

    /// Gets the integer representation of this ServiceFlags
    pub fn to_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for ServiceFlags {
    fn from(n: u64) -> Self {
        ServiceFlags(n)
    }
}

#[derive(Debug)]
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

impl VersionPayload {
    pub fn build(
        services: ServiceFlags,
        addr_recv_serv: ServiceFlags,
        addr_recv_socket: SocketAddr,
        addr_trans_serv: ServiceFlags,
        addr_trans_socket: SocketAddr,
        nonce: u64,
        start_height: i32,
        relay: bool,
    ) -> Payload {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("get timestamp since unix epoch")
            .as_secs() as i64;

        let (addr_recv, addr_recv_port) =
            VersionPayload::socket_to_octets_and_port(addr_recv_socket);

        let (addr_trans, addr_trans_port) =
            VersionPayload::socket_to_octets_and_port(addr_trans_socket);

        const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
        const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

        let user_agent = format!("{}:{}", CARGO_PKG_NAME, CARGO_PKG_VERSION).to_string();

        Payload::Version(VersionPayload {
            version: PROTOCOL_VERSION,
            services: services.to_u64(),
            timestamp,
            addr_recv_serv: addr_recv_serv.to_u64(),
            addr_recv,
            addr_recv_port,
            addr_trans_serv: addr_trans_serv.to_u64(),
            addr_trans,
            addr_trans_port,
            user_agent,
            nonce,
            start_height,
            relay,
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = vec![];
        buffer.write_i32::<LittleEndian>(self.version)?;
        buffer.write_u64::<LittleEndian>(self.services)?;
        buffer.write_i64::<LittleEndian>(self.timestamp)?;
        buffer.write_u64::<LittleEndian>(self.addr_recv_serv)?;
        buffer.write_u128::<BigEndian>(u128::from_ne_bytes(self.addr_recv))?;
        buffer.write_u16::<LittleEndian>(self.addr_recv_port)?;
        buffer.write_u64::<LittleEndian>(self.addr_trans_serv)?;
        buffer.write_u128::<BigEndian>(u128::from_ne_bytes(self.addr_trans))?;
        buffer.write_u16::<LittleEndian>(self.addr_trans_port)?;
        buffer.write_u64::<LittleEndian>(self.nonce)?;
        buffer.write_u8(self.user_agent.len() as u8)?;
        buffer.write_all(self.user_agent.as_bytes())?;
        buffer.write_i32::<LittleEndian>(self.start_height)?;
        buffer.write_u8(self.relay.into())?;
        Ok(buffer)
    }

    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self> {
        let version_payload = VersionPayload {
            version: bytes.read_i32::<LittleEndian>()?,
            services: bytes.read_u64::<LittleEndian>()?,
            timestamp: bytes.read_i64::<LittleEndian>()?,
            addr_recv_serv: bytes.read_u64::<LittleEndian>()?,
            addr_recv: bytes.read_u128::<BigEndian>()?.to_ne_bytes(),
            addr_recv_port: bytes.read_u16::<BigEndian>()?,
            addr_trans_serv: bytes.read_u64::<LittleEndian>()?,
            addr_trans: bytes.read_u128::<BigEndian>()?.to_ne_bytes(),
            addr_trans_port: bytes.read_u16::<BigEndian>()?,
            nonce: bytes.read_u64::<LittleEndian>()?,
            user_agent: {
                let mut tmp_bytes = vec![0u8; 0];
                let user_agent_len = bytes.read_u8()?;
                let user_agent_bytes = vec![0u8; user_agent_len as usize];
                bytes.read_exact(&mut tmp_bytes)?;
                String::from_utf8(user_agent_bytes)?
            },
            start_height: bytes.read_i32::<LittleEndian>()?,
            relay: bytes.read_u8()? != 0x00,
        };

        Ok(version_payload)
    }

    fn socket_to_octets_and_port(socket: SocketAddr) -> ([u8; 16], u16) {
        (
            match socket.ip() {
                std::net::IpAddr::V4(x) => x.to_ipv6_mapped(),
                std::net::IpAddr::V6(x) => x,
            }
            .octets(),
            socket.port(),
        )
    }
}
