use std::pin::Pin;

#[cfg(feature = "tracing")]
use tracing::instrument;

use async_trait::async_trait;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use reqwest::{Client, Url};

use crate::transport::Transport;
use crate::types::{HttpRequest, HttpResponse, HttpVerb};
use crate::{Error, Result};

/// A [`Transport`] implementation that uses the `reqwest` crate for making HTTP requests.
///
/// This is the default transport used by [`OllamaClient`](crate::OllamaClient) if no custom transport
/// is provided. It handles constructing `reqwest` clients, sending requests,
/// and processing responses, including streaming responses.
pub struct ReqwestTransport {
    client: Client,
    base_url: Url,
    api_key: Option<String>,
}

impl ReqwestTransport {
    /// Creates a new `ReqwestTransport`.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the Ollama server.
    /// * `api_key` - An optional API key for authentication.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Client`] if the `reqwest` client cannot be built.
    pub fn new(base_url: Url, api_key: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .build()
            .map_err(|e| Error::Client(e.to_string()))?;
        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    /// Helper to build and send a reqwest request, handling common logic.
    async fn build_and_send_request(&self, request: HttpRequest) -> Result<reqwest::Response> {
        let url = self
            .base_url
            .join(&request.url)
            .map_err(|e| Error::Client(e.to_string()))?;

        let mut request_builder = match request.verb {
            HttpVerb::GET => self.client.get(url),
            HttpVerb::POST => self.client.post(url),
            HttpVerb::PUT => self.client.put(url),
            HttpVerb::DELETE => self.client.delete(url),
        };

        if let Some(api_key) = &self.api_key {
            request_builder = request_builder.bearer_auth(api_key);
        }

        if let Some(body) = request.body {
            request_builder = request_builder.json(&body);
        }

        let response = request_builder.send().await.map_err(Error::Transport)?;
        response.error_for_status_ref().map_err(Error::Transport)?;
        Ok(response)
    }
}

#[async_trait]
impl Transport for ReqwestTransport {
    /// Sends a non-streaming HTTP request using `reqwest`.
    ///
    /// # Arguments
    ///
    /// * `request` - The [`HttpRequest`] to send.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Transport`] if the request fails or the response cannot be read.
    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    async fn send_http_request(&self, request: HttpRequest) -> Result<HttpResponse> {
        let response = self.build_and_send_request(request).await?;
        let response_bytes = response.bytes().await.map_err(Error::Transport)?;
        Ok(HttpResponse {
            body: Some(response_bytes),
        })
    }

    /// Sends a streaming HTTP request using `reqwest` and returns a stream of response bytes.
    ///
    /// # Arguments
    ///
    /// * `request` - The [`HttpRequest`] to send.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Transport`] if the request fails or the stream cannot be established.
    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    async fn send_http_stream_request(
        &self,
        request: HttpRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        let response = self.build_and_send_request(request).await?;
        let stream = response
            .bytes_stream()
            .map(|item| item.map_err(Error::Transport))
            .boxed();
        Ok(stream)
    }
}
