//! Contains data structures for requests and responses to the Ollama API.
//!
//! This module defines the various types used to interact with the Ollama server,
//! including chat messages, generation requests, model information, and shared utilities.

pub mod chat;
pub mod generate;
mod http;
mod models;
mod shared;

pub use http::*;
pub use models::*;
pub use shared::*;
