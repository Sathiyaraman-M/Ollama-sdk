use futures::stream::unfold;
use futures::StreamExt;

#[cfg(feature = "metrics")]
use metrics::counter;
#[cfg(feature = "tracing")]
use tracing::instrument;

use crate::builder::OllamaClientBuilder;
use crate::stream::chat_stream_parser::ChatStreamParser;
use crate::stream::generate_stream_parser::GenerateStreamParser;
use crate::tools::DynTool;
use crate::types::chat::{
    ChatRequest, ChatResponse, ChatStream, SimpleChatRequest, StreamingChatRequest,
};
use crate::types::generate::{
    GenerateRequest, GenerateResponse, GenerateStream, SimpleGenerateRequest,
    StreamingGenerateRequest,
};
use crate::types::models::{ListModelsResponse, ListRunningModelsResponse};
use crate::types::HttpRequest;
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

        let chat_request = ChatRequest::from(request);
        let request = HttpRequest::new("/api/chat").post().body(chat_request)?;

        let byte_stream = self.transport.send_http_stream_request(request).await?;
        let parser = ChatStreamParser::new(byte_stream);

        let response_stream = unfold(parser, |mut parser| async {
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

        let chat_request = ChatRequest::from(request);
        let request = HttpRequest::new("/api/chat").post().body(chat_request)?;

        let response = self.transport.send_http_request(request).await?;

        match response.body {
            Some(bytes) => ChatResponse::from_bytes(bytes),
            None => Err(Error::Protocol("Missing response body".into())),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    pub async fn generate_stream(
        &self,
        request: StreamingGenerateRequest,
    ) -> Result<GenerateStream> {
        #[cfg(feature = "metrics")]
        counter!("ollama_client.generate_requests_total", "type" => "streaming").increment(1);

        let generate_request = GenerateRequest::from(request);
        let request = HttpRequest::new("/api/generate")
            .post()
            .body(generate_request)?;

        let byte_stream = self.transport.send_http_stream_request(request).await?;
        let parser = GenerateStreamParser::new(byte_stream);

        let response_stream = unfold(parser, |mut parser| async {
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

        let generate_request = GenerateRequest::from(request);
        let request = HttpRequest::new("/api/generate")
            .post()
            .body(generate_request)?;

        let response = self.transport.send_http_request(request).await?;

        match response.body {
            Some(bytes) => GenerateResponse::from_bytes(bytes),
            None => Err(Error::Protocol("Missing response body".into())),
        }
    }

    pub async fn list_models(&self) -> Result<ListModelsResponse> {
        let request = HttpRequest::new("/api/tags");

        let response = self.transport.send_http_request(request).await?;

        match response.body {
            Some(bytes) => ListModelsResponse::from_bytes(bytes),
            None => Err(Error::Protocol("Missing response body".into())),
        }
    }

    pub async fn list_running_models(&self) -> Result<ListRunningModelsResponse> {
        let request = HttpRequest::new("/api/ps");

        let response = self.transport.send_http_request(request).await?;

        match response.body {
            Some(bytes) => ListRunningModelsResponse::from_bytes(bytes),
            None => Err(Error::Protocol("Missing response body".into())),
        }
    }
}
