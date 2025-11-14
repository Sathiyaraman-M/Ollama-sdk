use std::sync::Arc;
use std::time::Duration;

use futures::{StreamExt, TryStreamExt};
use reqwest::Url;

#[cfg(feature = "metrics")]
use metrics::counter;
#[cfg(feature = "tracing")]
use tracing::instrument;

use crate::stream::chat_stream_parser::ChatStreamParser;
use crate::stream::generate_stream_parser::GenerateStreamParser;
use crate::tools::registry::ToolRegistry;
use crate::tools::DynTool;
use crate::transport::reqwest_transport::ReqwestTransport;
use crate::transport::Transport;
use crate::types::chat::{ChatResponse, ChatStream, SimpleChatRequest, StreamingChatRequest};
use crate::types::generate::{
    GenerateResponse, GenerateStream, SimpleGenerateRequest, StreamingGenerateRequest,
};

#[derive(Clone)]
pub struct OllamaClient {
    transport: Arc<dyn Transport + Send + Sync>,
    tool_registry: ToolRegistry,
}
use crate::{Error, Result};

impl OllamaClient {
    pub fn builder() -> OllamaClientBuilder {
        OllamaClientBuilder {
            base_url: None,
            api_key: None,
            max_tool_runtime: None,
            tool_registry: ToolRegistry::new(),
            transport: None,
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(self, tool)))]
    pub fn register_tool(&mut self, tool: DynTool) -> Result<()> {
        self.tool_registry.register_tool(tool)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(self)))]
    pub fn unregister_tool(&mut self, name: &str) -> Result<()> {
        self.tool_registry.unregister_tool(name)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    pub async fn chat_stream(&self, request: StreamingChatRequest) -> Result<ChatStream> {
        #[cfg(feature = "metrics")]
        counter!("ollama_client.chat_requests_total", "type" => "streaming").increment(1);

        let byte_stream = self.transport.send_chat_request(request.into()).await?;
        let parser = ChatStreamParser::new(byte_stream);

        let response_stream = futures::stream::unfold(parser, |mut parser| async {
            parser.next().await.map(|e| (e, parser))
        });

        Ok(ChatStream {
            inner: Box::pin(response_stream),
        })
    }

    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    pub async fn chat_simple(&self, request: SimpleChatRequest) -> Result<ChatResponse> {
        #[cfg(feature = "metrics")]
        counter!("ollama_client.chat_requests_total", "type" => "non_streaming").increment(1);

        let response_bytes = self.transport.send_chat_request(request.into()).await?;

        // Collect all bytes from the stream
        let full_response_bytes = response_bytes
            .try_collect::<Vec<bytes::Bytes>>()
            .await
            .map_err(|e| Error::Client(e.to_string()))?
            .into_iter()
            .flatten()
            .collect::<Vec<u8>>();

        // Deserialize the full response
        serde_json::from_slice(&full_response_bytes)
            .map_err(|e| Error::Protocol(format!("Failed to deserialize chat response: {}", e)))
    }

    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    pub async fn generate_stream(
        &self,
        request: StreamingGenerateRequest,
    ) -> Result<GenerateStream> {
        #[cfg(feature = "metrics")]
        counter!("ollama_client.generate_requests_total", "type" => "streaming").increment(1);

        let byte_stream = self.transport.send_generate_request(request.into()).await?;
        let parser = GenerateStreamParser::new(byte_stream);

        let response_stream = futures::stream::unfold(parser, |mut parser| async {
            parser.next().await.map(|event| (event, parser))
        });

        Ok(GenerateStream {
            inner: Box::pin(response_stream),
        })
    }

    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    pub async fn generate_simple(
        &self,
        request: SimpleGenerateRequest,
    ) -> Result<GenerateResponse> {
        #[cfg(feature = "metrics")]
        counter!("ollama_client.generate_requests_total", "type" => "non_streaming").increment(1);

        let response_bytes = self.transport.send_generate_request(request.into()).await?;

        // Collect all bytes from the stream
        let full_response_bytes = response_bytes
            .try_collect::<Vec<bytes::Bytes>>()
            .await
            .map_err(|e| Error::Client(e.to_string()))?
            .into_iter()
            .flatten()
            .collect::<Vec<u8>>();

        // Deserialize the full response
        serde_json::from_slice(&full_response_bytes)
            .map_err(|e| Error::Protocol(format!("Failed to deserialize generate response: {}", e)))
    }
}

// OllamaClientBuilder
pub struct OllamaClientBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
    max_tool_runtime: Option<Duration>,
    tool_registry: ToolRegistry,
    transport: Option<Arc<dyn Transport + Send + Sync>>,
}

impl OllamaClientBuilder {
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn max_tool_runtime(mut self, duration: Duration) -> Self {
        self.max_tool_runtime = Some(duration);
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
