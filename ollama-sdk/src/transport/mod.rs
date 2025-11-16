//! Defines the [`Transport`] trait and its implementations for making HTTP requests to the Ollama API.
//!
//! This module provides an abstraction layer for sending HTTP requests, allowing
//! different underlying HTTP clients or mock implementations to be used.

use std::pin::Pin;

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;

use crate::types::{HttpRequest, HttpResponse};
use crate::Result;

mod mock_transport;
mod reqwest_transport;

pub use mock_transport::MockTransport;
pub use reqwest_transport::ReqwestTransport;

/// A trait for sending HTTP requests to the Ollama API.
///
/// Implementations of this trait handle the actual network communication.
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    /// Sends a non-streaming HTTP request and returns the complete response.
    ///
    /// # Arguments
    ///
    /// * `request` - The [`HttpRequest`] to send.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](enum@crate::Error) if the request fails or the response cannot be processed.
    async fn send_http_request(&self, request: HttpRequest) -> Result<HttpResponse>;

    /// Sends a streaming HTTP request and returns a stream of response bytes.
    ///
    /// This is used for API endpoints that return a continuous stream of data,
    /// such as chat completions or text generation.
    ///
    /// # Arguments
    ///
    /// * `request` - The [`HttpRequest`] to send.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](enum@crate::Error) if the request fails or the stream cannot be established.
    async fn send_http_stream_request(
        &self,
        request: HttpRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>>;
}
