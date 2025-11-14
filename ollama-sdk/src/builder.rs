use std::sync::Arc;

#[cfg(feature = "tracing")]
use tracing::instrument;

use reqwest::Url;

use crate::tools::registry::ToolRegistry;
use crate::transport::reqwest_transport::ReqwestTransport;
use crate::transport::Transport;
use crate::OllamaClient;
use crate::{Error, Result};

pub struct OllamaClientBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
    tool_registry: ToolRegistry,
    transport: Option<Arc<dyn Transport + Send + Sync>>,
}

impl OllamaClientBuilder {
    pub(crate) fn new() -> Self {
        OllamaClientBuilder {
            base_url: None,
            api_key: None,
            tool_registry: ToolRegistry::new(),
            transport: None,
        }
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn tool_registry(mut self, registry: ToolRegistry) -> Self {
        self.tool_registry = registry;
        self
    }

    pub fn transport(mut self, transport: Arc<dyn Transport + Send + Sync>) -> Self {
        self.transport = Some(transport);
        self
    }

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
