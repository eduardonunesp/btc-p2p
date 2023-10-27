use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{Read, Write},
    net::SocketAddr,
    time::SystemTime,
};

use super::{command::Command, errors::Result, PROTOCOL_VERSION};

/// Payload represents the payload of a message
/// The inner type encapsulates all the different payloads
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Version(VersionPayload),
    VerAck,
    Ping(u64),
    Pong(u64),
    Empty,
}

impl Payload {
    /// to_bytes converts the payload to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Payload::Version(version_payload) => Ok(version_payload.to_bytes()?.to_vec()),
            Payload::VerAck => Ok(vec![]),
            Payload::Ping(nonce) => Ok(nonce.to_le_bytes().to_vec()),
            Payload::Pong(nonce) => Ok(nonce.to_le_bytes().to_vec()),
            Payload::Empty => Ok(vec![]),
        }
    }

    /// from_bytes converts bytes to a payload
    /// the command is needed to determine the payload type
    pub fn from_bytes(command: &Command, bytes: &[u8]) -> Result<Self> {
        match command {
            Command::Version => Ok(Payload::Version(VersionPayload::from_bytes(&bytes)?)),
            Command::VerAck => Ok(Payload::VerAck),
            Command::Ping => Ok(Payload::Ping(u64::from_le_bytes(bytes.try_into()?))),
            Command::Pong => Ok(Payload::Pong(u64::from_le_bytes(bytes.try_into()?))),
        }
    }
}

/// ServiceFlags represents the service flags of a node
/// https://developer.bitcoin.org/reference/p2p_networking.html#version
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

    /// Gets the ServiceFlags from an integer representation
    pub fn from_u64(n: u64) -> Self {
        ServiceFlags(n)
    }
}

impl From<u64> for ServiceFlags {
    fn from(n: u64) -> Self {
        ServiceFlags(n)
    }
}

/// VersionPayload represents the payload of a version message
/// https://developer.bitcoin.org/reference/p2p_networking.html#version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionPayload {
    /// The highest protocol version understood by the transmitting node.
    pub version: i32,

    /// The services supported by the transmitting node encoded as a bitfield.
    pub services: u64,

    /// The current Unix epoch time according to the transmitting node’s clock.
    pub timestamp: i64,

    /// The services supported by the receiving node as perceived by the transmitting node. Same format as the ‘services’ field above.
    pub addr_recv_serv: u64,

    /// The IPv6 address of the receiving node as perceived by the transmitting node in big endian byte order.
    pub addr_recv: [u8; 16],

    /// The port number of the receiving node as perceived by the transmitting node in big endian byte order.
    pub addr_recv_port: u16,

    /// Added inprotocol version 106. The services supported by the transmitting node. Should be identical to the ‘services’ field above.
    pub addr_trans_serv: u64,

    /// Added inprotocol version 106. The IPv6 address of the transmitting node in big endian byte order.
    pub addr_trans: [u8; 16],

    /// Added inprotocol version 106. The port number of the transmitting node in big endian byte order.
    pub addr_trans_port: u16,

    /// Added inprotocol version 106. A random nonce which can help a node detect a connection to itself.
    /// If the nonce is 0, the nonce field is ignored.
    /// If the nonce is anything else, a node should terminate the connection on receipt of a “version” message with a nonce it previously sent.
    pub nonce: u64,

    /// Added inprotocol version 106. Number of bytes in following user_agent field. If 0x00, no user agent field is sent.
    pub user_agent: String,

    /// Added inprotocol version 209. The height of the transmitting node’s best block chain or, in the case of an SPV client, best block header chain.
    pub start_height: i32,

    /// Added inprotocol version 70001as described byBIP37.
    /// Transaction relay flag. If 0x00, no “inv” messages or “tx” messages announcing new transactions should be sent to this client until it sends a “filterload” message or “filterclear” message.
    /// If the relay field is not present or is set to 0x01, this node wants “inv” messages and “tx” messages announcing new transactions.
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

        let user_agent = format!("/{}:{}/", CARGO_PKG_NAME, CARGO_PKG_VERSION).to_string();

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

    /// to_bytes converts the payload to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = vec![];
        buffer.write_i32::<LittleEndian>(self.version)?;
        buffer.write_u64::<LittleEndian>(self.services)?;
        buffer.write_i64::<LittleEndian>(self.timestamp)?;
        buffer.write_u64::<LittleEndian>(self.addr_recv_serv)?;
        buffer.write_u128::<BigEndian>(u128::from_ne_bytes(self.addr_recv))?;
        buffer.write_u16::<BigEndian>(self.addr_recv_port)?;
        buffer.write_u64::<LittleEndian>(self.addr_trans_serv)?;
        buffer.write_u128::<BigEndian>(u128::from_ne_bytes(self.addr_trans))?;
        buffer.write_u16::<BigEndian>(self.addr_trans_port)?;
        buffer.write_u64::<LittleEndian>(self.nonce)?;
        buffer.write_u8(self.user_agent.len() as u8)?;
        buffer.write_all(self.user_agent.as_bytes())?;
        buffer.write_i32::<LittleEndian>(self.start_height)?;
        buffer.write_u8(self.relay.into())?;
        Ok(buffer)
    }

    /// from_bytes converts bytes to a payload
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

    /// socket_to_octets_and_port converts a socket address (SocketAddr) to its octets and port ([u8; 16], u16)
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

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, TestResult};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for Payload {
        fn arbitrary(g: &mut quickcheck::Gen) -> Payload {
            VersionPayload::build(
                ServiceFlags::from_u64(u64::arbitrary(g)),
                ServiceFlags::from_u64(u64::arbitrary(g)),
                SocketAddr::arbitrary(g),
                ServiceFlags::from_u64(u64::arbitrary(g)),
                SocketAddr::arbitrary(g),
                u64::arbitrary(g),
                i32::arbitrary(g),
                bool::arbitrary(g),
            )
        }
    }

    impl Arbitrary for VersionPayload {
        fn arbitrary(g: &mut quickcheck::Gen) -> VersionPayload {
            VersionPayload {
                version: i32::arbitrary(g),
                services: u64::arbitrary(g),
                timestamp: i64::arbitrary(g),
                addr_recv_serv: u64::arbitrary(g),
                addr_recv: [u8::arbitrary(g); 16],
                addr_recv_port: u16::arbitrary(g),
                addr_trans_serv: u64::arbitrary(g),
                addr_trans: [u8::arbitrary(g); 16],
                addr_trans_port: u16::arbitrary(g),
                nonce: u64::arbitrary(g),
                user_agent: "".to_string(),
                start_height: i32::arbitrary(g),
                relay: bool::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn payload_from_bytes(payload: Payload) {
        let mut nonce = 0;

        if let Payload::Version(version_payload) = &payload {
            nonce = version_payload.nonce;
        }

        let bytes = payload.to_bytes().unwrap();
        let result = Payload::from_bytes(&Command::Version, &bytes[..]).unwrap();

        if let Payload::Version(version_payload) = result {
            assert_eq!(version_payload.nonce, nonce);
        }
    }

    #[quickcheck]
    fn check_protocol_version(payload: Payload) -> TestResult {
        match payload {
            Payload::Version(version_payload) => {
                if version_payload.version == PROTOCOL_VERSION {
                    TestResult::passed()
                } else {
                    TestResult::failed()
                }
            }
            _ => TestResult::discard(),
        }
    }

    #[quickcheck]
    fn version_data_from_bytes(version_payload: VersionPayload) {
        let bytes = version_payload.to_bytes().unwrap();
        let _ = VersionPayload::from_bytes(&bytes).unwrap();
    }
}
