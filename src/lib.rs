#![deny(unused_must_use, rust_2018_idioms)]

//! Bitcoin p2p protocol implementation
//!
//! This crate provides a pure Rust implementation of the Bitcoin protocol.

mod command;
mod errors;
mod message;
mod network;
mod payload;

pub use command::Command;
pub use errors::{BTCP2PError, Result};
pub use message::Message;
pub use network::Network;
pub use payload::{Payload, ServiceFlags, VersionPayload};

/// Protocol version for the BTC proto
/// https://developer.bitcoin.org/reference/p2p_networking.html#protocol-versions
///
/// The table below lists some notable versions of the P2P network protocol, with the most recent versions listed first.
/// (If you know of a protocol version that implemented a major change but which is not listed here, please open an issue.)
const PROTOCOL_VERSION: i32 = 70015;

/// Message format for the BTC proto:
/// https://developer.bitcoin.org/reference/p2p_networking.html#message-headers

/// Max size for the start string in the message header
const START_STRING_SIZE: usize = 4;

/// Max size for the command name in the message header
const COMMAND_NAME_SIZE: usize = 12;

/// Size of the payload length which is a uint32 in the message header
const PAYLOAD_LEN_SIZE: usize = 4;

/// Size of the checksum in the message header
const CHECKSUM_SIZE: usize = 4;

/// Total size of the message header and max payload size
const HEADER_SIZE: usize = START_STRING_SIZE + COMMAND_NAME_SIZE + PAYLOAD_LEN_SIZE + CHECKSUM_SIZE;

// 32 MB
const MAX_PAYLOAD_SIZE: usize = 32 * 1024 * 1024;

/// Position of the start string in the message header
const HEADER_START_STRING_RANGE: std::ops::Range<usize> = 0..4;

/// Position of the command name in the message header
const HEADER_COMMAND_NAME_RANGE: std::ops::Range<usize> = 4..16;

/// Position of the payload length in the message header
const HEADER_PAYLOAD_LEN_RANGE: std::ops::Range<usize> = 16..20;

/// Position of the checksum in the message header
const HEADER_CHECKSUM_RANGE: std::ops::Range<usize> = 20..24;
