use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures::{Stream, StreamExt};
use reqwest::Url;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::errors::{Error, Result};
use crate::stream::parser::StreamParser;
use crate::tools::registry::ToolRegistry;
use crate::tools::{DynTool, ToolContext};
use crate::transport::reqwest_transport::ReqwestTransport;
use crate::transport::Transport;
use crate::types::{ChatRequest, StreamEvent};

#[derive(Clone)]
pub struct OllamaClient {
    transport: Arc<dyn Transport + Send + Sync>,
    tool_registry: ToolRegistry,
    max_tool_runtime: Duration,
}

impl OllamaClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let base_url = Url::parse(&base_url.into())
            .map_err(|e| Error::Client(format!("Invalid base URL: {}", e)))?;
        let transport = ReqwestTransport::new(base_url, None)?;
        Ok(Self {
            transport: Arc::new(transport),
            tool_registry: ToolRegistry::new(),
            max_tool_runtime: Duration::from_secs(30), // Default to 30 seconds
        })
    }

    pub fn new_from_env() -> Result<Self> {
        let base_url =
            std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
        let api_key = std::env::var("OLLAMA_API_KEY").ok();

        let base_url = Url::parse(&base_url)
            .map_err(|e| Error::Client(format!("Invalid OLLAMA_HOST URL: {}", e)))?;
        let transport = ReqwestTransport::new(base_url, api_key)?;
        Ok(Self {
            transport: Arc::new(transport),
            tool_registry: ToolRegistry::new(),
            max_tool_runtime: Duration::from_secs(30), // Default to 30 seconds
        })
    }

    pub fn register_tool(&mut self, tool: DynTool) -> Result<()> {
        self.tool_registry.register_tool(tool)
    }

    pub fn unregister_tool(&mut self, name: &str) -> Result<()> {
        self.tool_registry.unregister_tool(name)
    }

    pub async fn chat_stream(&self, mut request: ChatRequest) -> Result<ChatStream> {
        request.stream = Some(true); // Ensure streaming is enabled
        let byte_stream = self.transport.send_chat_request(request).await?;
        let parser = StreamParser::new(byte_stream);

        let client_arc = Arc::new(self.clone()); // Clone client for tool dispatching

        let max_tool_runtime = self.max_tool_runtime;

        let stream_with_dispatch = futures::stream::unfold(
            (parser, client_arc),
            move |(mut parser, client_arc)| async move {
                match parser.next().await {
                    Some(Ok(StreamEvent::ToolCall {
                        invocation_id,
                        name,
                        input,
                    })) => {
                        let client_for_tool = client_arc.clone();
                        let tool_registry_for_tool = client_arc.tool_registry.clone();
                        let cancellation_token = CancellationToken::new();

                        // Clone before moving into spawn
                        let invocation_id_clone = invocation_id.clone();
                        let name_clone = name.clone();
                        let input_clone = input.clone();

                        tokio::spawn(async move {
                            let tool_result = if let Some(tool) =
                                tool_registry_for_tool.get_tool(&name_clone)
                            {
                                let ctx = ToolContext {
                                    cancellation_token: cancellation_token.clone(),
                                };
                                match timeout(max_tool_runtime, tool.call(input_clone, ctx)).await {
                                    Ok(Ok(result)) => result,
                                    Ok(Err(e)) => serde_json::json!({"error": e.to_string()}),
                                    Err(_) => {
                                        serde_json::json!({"error": format!("Tool '{}' timed out after {:?}", name_clone, max_tool_runtime)})
                                    }
                                }
                            } else {
                                serde_json::json!({"error": format!("Tool '{}' not found", name_clone)})
                            };

                            if let Err(e) = client_for_tool
                                .send_tool_result(&invocation_id_clone, tool_result)
                                .await
                            {
                                eprintln!("Failed to send tool result: {:?}", e);
                            }
                        });

                        Some((
                            Ok(StreamEvent::ToolCall {
                                invocation_id,
                                name,
                                input,
                            }),
                            (parser, client_arc),
                        ))
                    }
                    Some(event) => Some((event, (parser, client_arc))),
                    None => None,
                }
            },
        );

        Ok(ChatStream {
            inner: Box::pin(stream_with_dispatch),
        })
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
    inner: Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>,
}

impl Stream for ChatStream {
    type Item = Result<StreamEvent>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}
