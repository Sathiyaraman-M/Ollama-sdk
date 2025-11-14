use thiserror::Error;

pub mod client;
pub mod stream;
pub mod tools;
pub mod transport;
pub mod types;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client error: {0}")]
    Client(String),

    #[error("Transport error: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Stream cancelled")]
    Cancelled,
}
