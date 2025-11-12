use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures::{Stream, StreamExt, TryStreamExt};
use reqwest::Url;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "metrics")]
use metrics::counter;
#[cfg(feature = "tracing")]
use tracing::{error, instrument};

use crate::errors::{Error, Result};
use crate::stream::parser::StreamParser;
use crate::tools::registry::ToolRegistry;
use crate::tools::{DynTool, ToolContext};
use crate::transport::reqwest_transport::ReqwestTransport;
use crate::transport::Transport;
use crate::types::chat::{ChatResponse, SimpleChatRequest, StreamEvent, StreamingChatRequest};

#[derive(Clone)]
pub struct OllamaClient {
    transport: Arc<dyn Transport + Send + Sync>,
    tool_registry: ToolRegistry,
    max_tool_runtime: Duration,
}

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
                            #[cfg(feature = "metrics")]
                            counter!("ollama_client.tool_calls_total").increment(1);

                            let tool_result = if let Some(tool) =
                                tool_registry_for_tool.get_tool(&name_clone)
                            {
                                let ctx = ToolContext {
                                    cancellation_token: cancellation_token.clone(),
                                };
                                match timeout(max_tool_runtime, tool.call(input_clone, ctx)).await {
                                    Ok(Ok(result)) => {
                                        #[cfg(feature = "metrics")]
                                        counter!("ollama_client.tool_call_successes_total")
                                            .increment(1);
                                        result
                                    }
                                    Ok(Err(e)) => {
                                        #[cfg(feature = "metrics")]
                                        counter!("ollama_client.tool_call_failures_total", "reason" => "tool_error").increment(1);
                                        serde_json::json!({"error": e.to_string()})
                                    }
                                    Err(_) => {
                                        #[cfg(feature = "metrics")]
                                        counter!("ollama_client.tool_call_failures_total", "reason" => "timeout").increment(1);
                                        serde_json::json!({"error": format!("Tool '{}' timed out after {:?}", name_clone, max_tool_runtime)})
                                    }
                                }
                            } else {
                                #[cfg(feature = "metrics")]
                                counter!("ollama_client.tool_call_failures_total", "reason" => "tool_not_found").increment(1);
                                serde_json::json!({"error": format!("Tool '{}' not found", name_clone)})
                            };

                            if let Err(e) = client_for_tool
                                .send_tool_result(&invocation_id_clone, tool_result)
                                .await
                            {
                                #[cfg(feature = "tracing")]
                                error!("Failed to send tool result: {:?}", e);
                                #[cfg(not(feature = "tracing"))]
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

    #[cfg_attr(feature = "tracing", instrument(skip(self)))]
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
            max_tool_runtime: self.max_tool_runtime.unwrap_or(Duration::from_secs(30)),
        })
    }
}
