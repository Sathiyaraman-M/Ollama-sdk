use std::sync::Arc;

#[cfg(feature = "tracing")]
use tracing::instrument;

use reqwest::Url;

use crate::tools::ToolRegistry;
use crate::transport::{ReqwestTransport, Transport};
use crate::{Error, OllamaClient, Result};

/// A builder for constructing an [`OllamaClient`].
///
/// This builder allows for flexible configuration of the client, including
/// the base URL of the Ollama server, an API key, a custom tool registry,
/// and a custom transport layer.
///
/// - Uses either `OLLAMA_HOST` environment variable or `http://127.0.0.1:11434`.
/// - Uses either `OLLAMA_API_KEY` environment variable or nothing.
/// - Starts with an empty [`ToolRegistry`] which can be populated later through [`OllamaClient`].
/// - Uses `reqwest`-based transport by default - [`ReqwestTransport`].
pub struct OllamaClientBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
    tool_registry: ToolRegistry,
    transport: Option<Arc<dyn Transport + Send + Sync>>,
}

impl OllamaClientBuilder {
    /// Creates a new [`OllamaClientBuilder`]. This method is called by [`OllamaClient::builder`]
    pub(crate) fn new() -> Self {
        OllamaClientBuilder {
            base_url: None,
            api_key: None,
            tool_registry: ToolRegistry::new(),
            transport: None,
        }
    }

    /// Sets the base URL for the Ollama API.
    ///
    /// If not set, the builder will try to read from the `OLLAMA_HOST` environment variable,
    /// defaulting to `http://127.0.0.1:11434` if the environment variable is not found.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Sets the API key for authentication with the Ollama API.
    ///
    /// If not set, the builder will try to read from the `OLLAMA_API_KEY` environment variable.
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets a custom [`ToolRegistry`] for the client.
    ///
    /// If not set, a default empty [`ToolRegistry`] will be used.
    /// Note that if you want to register/unregister tools after constructing
    /// [`OllamaClient`], you can use [`OllamaClient::register_tool`], and
    /// [`OllamaClient::unregister_tool`] to do so.
    ///
    /// The current method help you to setup a pre-filled [`ToolRegistry`].
    pub fn tool_registry(mut self, registry: ToolRegistry) -> Self {
        self.tool_registry = registry;
        self
    }

    /// Sets a custom transport implementation for the client.
    ///
    /// This allows for using different HTTP clients or mock implementations for testing.
    /// If not set, a `reqwest`-based transport \([`ReqwestTransport`]\) will be used.
    ///
    /// For testing, you can use [`MockTransport`](crate::transport::MockTransport)
    /// or your own mock [`Transport`] implementations.
    pub fn transport(mut self, transport: Arc<dyn Transport + Send + Sync>) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Builds the [`OllamaClient`] with the configured options.
    ///
    /// If no transport is provided, it constructs a default `reqwest`-based transport
    /// using the configured [`base_url`](OllamaClientBuilder::base_url) and
    /// [`api_key`](OllamaClientBuilder::api_key).
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Client`](variant@Error::Client) if the base URL is invalid or if there's an issue
    /// initializing [`ReqwestTransport`].
    #[cfg_attr(feature = "tracing", instrument(skip(self)))]
    pub fn build(self) -> Result<OllamaClient> {
        let transport = if let Some(t) = self.transport {
            t
        } else {
            let base_url_str = self.base_url.unwrap_or_else(|| {
                std::env::var("OLLAMA_HOST")
                    .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string())
            });
            let api_key = self
                .api_key
                .or_else(|| std::env::var("OLLAMA_API_KEY").ok());

            let base_url = Url::parse(&base_url_str)
                .map_err(|e| Error::Client(format!("Invalid base URL: {}", e)))?;

            Arc::new(ReqwestTransport::new(base_url, api_key)?)
        };

        Ok(OllamaClient {
            transport,
            tool_registry: self.tool_registry,
        })
    }
}
