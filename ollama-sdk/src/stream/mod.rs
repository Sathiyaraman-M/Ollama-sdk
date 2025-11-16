//! Provides utilities for parsing and handling streaming responses from the Ollama API.
//!
//! This module contains parsers for different types of streaming events,
//! such as chat completions and text generation.

mod chat_stream_parser;
mod generate_stream_parser;

pub use chat_stream_parser::ChatStreamParser;
pub use generate_stream_parser::GenerateStreamParser;
