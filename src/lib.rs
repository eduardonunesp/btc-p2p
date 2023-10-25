#![allow(unused)]
// #![warn(unused_crate_dependencies)]
// #![deny(unused_must_use, rust_2018_idioms)]

//! Bitcoin p2p protocol implementation
//!
//! This crate provides a pure Rust implementation of the Bitcoin protocol.

mod command;
mod encode;
mod errors;
mod message;
mod network;
mod payload;

pub use command::Command;
pub use errors::{BTCP2PError, Result};
pub use message::Message;
pub use network::Network;
pub use payload::{Payload, ServiceFlags, VersionPayload};
