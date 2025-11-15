use std::sync::Arc;

use thiserror::Error;

use self::tools::registry::ToolRegistry;
use self::transport::Transport;

pub mod builder;
pub mod client;
pub mod stream;
pub mod tools;
pub mod transport;
pub mod types;

#[derive(Clone)]
pub struct OllamaClient {
    transport: Arc<dyn Transport + Send + Sync>,
    tool_registry: ToolRegistry,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client error: {0}")]
    Client(String),

    #[error("Transport error: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("Server error: {0}")]
    Server(String),

    #[error("JSON error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Stream cancelled")]
    Cancelled,
}
