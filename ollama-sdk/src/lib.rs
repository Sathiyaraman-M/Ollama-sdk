//! A Rust SDK for interacting with the Ollama API.
//!
//! This crate provides a convenient and type-safe way to communicate with an Ollama server,
//! allowing you to interact with various language models, perform chat completions,
//! generate text, and manage models.
//!
//! ## Features
//!
//! - **Idiomatic Rust API:** Designed with Rust's best practices in mind.
//! - **Streaming support**: Handle streaming responses for chat and generate operations efficiently.
//! - **Configurable Transport:** Uses `reqwest` by default, with an extensible [`Transport`](crate::transport::Transport) trait for custom implementations.
//! - **Robust Error Handling:** Comprehensive error types for predictable error management.
//! - **Observability:** Optional `tracing` for detailed logging and `metrics` for performance monitoring.
//! - **Tooling Integration**: Support for tool definitions and registry.
//!
//! ## Getting Started
//!
//! To use this SDK, add `ollama-sdk` to your `Cargo.toml`.
//!
//! Check `examples` directory in GitHub for example usages.

use thiserror::Error;

mod builder;
mod client;
pub mod stream;
pub mod tools;
pub mod transport;
pub mod types;

pub use crate::{builder::OllamaClientBuilder, client::OllamaClient};

/// An alias for [`std::result::Result<T, E>`] where E is [`enum@Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Represents errors that can occur when interacting with the Ollama API.
#[derive(Error, Debug)]
pub enum Error {
    /// A client-side error, typically due to invalid input or configuration.
    #[error("Client error: {0}")]
    Client(String),

    /// An error originating from the underlying HTTP transport layer (e.g., network issues).
    #[error("Transport error: {0}")]
    Transport(#[from] reqwest::Error),

    /// An error returned by the Ollama server.
    #[error("Server error: {0}")]
    Server(String),

    /// An error during JSON serialization or deserialization.
    #[error("JSON error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// An error related to the Ollama API protocol (e.g., unexpected response format).
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// An error related to tool execution or definition.
    #[error("Tool error: {0}")]
    Tool(String),

    /// The streaming operation was cancelled.
    #[error("Stream cancelled")]
    Cancelled,
}
