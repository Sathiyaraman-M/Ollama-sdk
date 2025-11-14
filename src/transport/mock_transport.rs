use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[cfg(feature = "tracing")]
use tracing::instrument;

use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::{self};
use futures::Stream;
use futures::StreamExt;

use crate::errors::{Error, Result};
use crate::transport::Transport;
use crate::types::chat::{ChatRequest, ChatResponse, ChatStreamEvent};
use crate::types::generate::GenerateRequest;

#[derive(Clone, Default)]
pub struct MockTransport {
    chat_responses: Arc<Mutex<Vec<ChatStreamEvent>>>,
    raw_chat_responses: Arc<Mutex<Vec<String>>>, // Added for raw JSON strings
    non_streaming_response: Arc<Mutex<Option<ChatResponse>>>, // Added for non-streaming
    tool_results_sent: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_chat_responses(self, responses: Vec<ChatStreamEvent>) -> Self {
        *self.chat_responses.lock().unwrap() = responses;
        self
    }

    pub fn with_streaming_raw_responses(self, responses: Vec<String>) -> Self {
        *self.raw_chat_responses.lock().unwrap() = responses;
        self
    }

    pub fn with_non_streaming_response(self, response: ChatResponse) -> Self {
        *self.non_streaming_response.lock().unwrap() = Some(response);
        self
    }

    pub fn get_tool_results_sent(&self) -> Vec<(String, serde_json::Value)> {
        self.tool_results_sent.lock().unwrap().clone()
    }
}

#[async_trait]
impl Transport for MockTransport {
    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    async fn send_generate_request(
        &self,
        request: GenerateRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        if request.stream {
            let responses = self
                .chat_responses
                .lock()
                .unwrap()
                .drain(..)
                .collect::<Vec<_>>();
            let byte_stream = stream::iter(responses)
                .map(|event| {
                    let json_string = serde_json::to_string(&event).map_err(|e| {
                        Error::Protocol(format!("Failed to serialize mock event: {}", e))
                    })?;
                    Ok(Bytes::from(format!("{}\n", json_string)))
                })
                .boxed();
            Ok(byte_stream)
        } else {
            let response = self
                .non_streaming_response
                .lock()
                .unwrap()
                .take()
                .ok_or_else(|| {
                    Error::Protocol(
                        "MockTransport: No non-streaming response configured".to_string(),
                    )
                })?;

            let json_string = serde_json::to_string(&response).map_err(|e| {
                Error::Protocol(format!("Failed to serialize mock generate response: {}", e))
            })?;
            let byte_stream = stream::once(async { Ok(Bytes::from(json_string)) }).boxed();
            Ok(byte_stream)
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    async fn send_chat_request(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        if request.stream.unwrap_or(false) {
            let raw_responses = self
                .raw_chat_responses
                .lock()
                .unwrap()
                .drain(..)
                .collect::<Vec<_>>();
            if !raw_responses.is_empty() {
                let byte_stream = stream::iter(raw_responses)
                    .map(|s| Ok(Bytes::from(format!("{}\n", s))))
                    .boxed();
                Ok(byte_stream)
            } else {
                let responses = self
                    .chat_responses
                    .lock()
                    .unwrap()
                    .drain(..)
                    .collect::<Vec<_>>();
                let byte_stream = stream::iter(responses)
                    .map(|event| {
                        let json_string = serde_json::to_string(&event).map_err(|e| {
                            Error::Protocol(format!("Failed to serialize mock event: {}", e))
                        })?;
                        Ok(Bytes::from(format!("{}\n", json_string)))
                    })
                    .boxed();
                Ok(byte_stream)
            }
        } else {
            let response = self
                .non_streaming_response
                .lock()
                .unwrap()
                .take()
                .ok_or_else(|| {
                    Error::Protocol(
                        "MockTransport: No non-streaming response configured".to_string(),
                    )
                })?;

            let json_string = serde_json::to_string(&response).map_err(|e| {
                Error::Protocol(format!("Failed to serialize mock chat response: {}", e))
            })?;
            let byte_stream = stream::once(async { Ok(Bytes::from(json_string)) }).boxed();
            Ok(byte_stream)
        }
    }
}
