use thiserror::Error;

pub type Result<T> = std::result::Result<T, BTCP2PError>;

///! BTCP2PError represents an error in the BTC proto
#[derive(Debug, Error)]
pub enum BTCP2PError {
    #[error("Unknown network")]
    UnknowNetwork,

    #[error("Failed to read or write buffer")]
    BufferIOError(#[from] std::io::Error),

    #[error("Invalid header size")]
    InvalidHeaderSize,

    #[error("Invalid payload size")]
    PayloadTooLarge,

    #[error("Invalid checksum")]
    InvalidChecksum,

    #[error("Invalid command")]
    InvalidCommand,

    #[error("Failed on decode bytes")]
    DecodeError(#[from] std::array::TryFromSliceError),

    #[error("Failed to decode command")]
    DecodeCommandError(#[from] std::string::FromUtf8Error),
}
