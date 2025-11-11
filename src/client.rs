use std::sync::Arc;

use futures::{Stream, StreamExt};
use reqwest::Url;

use crate::errors::{Error, Result};
use crate::stream::parser::StreamParser;
use crate::transport::reqwest_transport::ReqwestTransport;
use crate::transport::Transport;
use crate::types::{ChatRequest, StreamEvent};

pub struct OllamaClient {
    transport: Arc<dyn Transport + Send + Sync>,
}

impl OllamaClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let base_url = Url::parse(&base_url.into())
            .map_err(|e| Error::Client(format!("Invalid base URL: {}", e)))?;
        let transport = ReqwestTransport::new(base_url, None)?;
        Ok(Self {
            transport: Arc::new(transport),
        })
    }

    pub fn new_from_env() -> Result<Self> {
        let base_url = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
        let api_key = std::env::var("OLLAMA_API_KEY").ok();

        let base_url = Url::parse(&base_url)
            .map_err(|e| Error::Client(format!("Invalid OLLAMA_HOST URL: {}", e)))?;
        let transport = ReqwestTransport::new(base_url, api_key)?;
        Ok(Self {
            transport: Arc::new(transport),
        })
    }

    pub async fn chat_stream(
        &self,
        mut request: ChatRequest,
    ) -> Result<ChatStream> {
        request.stream = Some(true); // Ensure streaming is enabled
        let byte_stream = self.transport.send_chat_request(request).await?;
        let parser = StreamParser::new(byte_stream);
        Ok(ChatStream { inner: Box::new(parser) })
    }

    pub async fn send_tool_result(
        &self,
        invocation_id: &str,
        result: serde_json::Value,
    ) -> Result<()> {
        self.transport.send_tool_result(invocation_id, result).await
    }
}

// ChatStream type
pub struct ChatStream {
    inner: Box<dyn Stream<Item = Result<StreamEvent>> + Send + Unpin>,
}

impl Stream for ChatStream {
    type Item = Result<StreamEvent>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next_unpin(cx)
    }
}
