use futures::{StreamExt, TryStreamExt};

#[cfg(feature = "metrics")]
use metrics::counter;
#[cfg(feature = "tracing")]
use tracing::instrument;

use crate::builder::OllamaClientBuilder;
use crate::stream::chat_stream_parser::ChatStreamParser;
use crate::stream::generate_stream_parser::GenerateStreamParser;
use crate::tools::DynTool;
use crate::types::chat::{ChatResponse, ChatStream, SimpleChatRequest, StreamingChatRequest};
use crate::types::generate::{
    GenerateResponse, GenerateStream, SimpleGenerateRequest, StreamingGenerateRequest,
};
use crate::OllamaClient;
use crate::{Error, Result};

impl OllamaClient {
    pub fn builder() -> OllamaClientBuilder {
        OllamaClientBuilder::new()
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
