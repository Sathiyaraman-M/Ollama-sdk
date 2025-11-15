use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[cfg(feature = "tracing")]
use tracing::instrument;

use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::{self};
use futures::Stream;
use futures::StreamExt;

use crate::transport::Transport;
use crate::types::chat::ChatStreamEvent;
use crate::types::{HttpRequest, HttpResponse};
use crate::{Error, Result};

#[derive(Clone, Default)]
pub struct MockTransport {
    // For streaming responses (chat/generate)
    chat_stream_events: Arc<Mutex<Vec<ChatStreamEvent>>>,
    generate_stream_bytes: Arc<Mutex<Vec<Bytes>>>,
    raw_chat_stream_strings: Arc<Mutex<Vec<String>>>,

    // For non-streaming responses
    non_streaming_http_response: Arc<Mutex<Option<HttpResponse>>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_chat_stream_events(self, events: Vec<ChatStreamEvent>) -> Self {
        *self.chat_stream_events.lock().unwrap() = events;
        self
    }

    pub fn with_generate_stream_bytes(self, bytes: Vec<Bytes>) -> Self {
        *self.generate_stream_bytes.lock().unwrap() = bytes;
        self
    }

    pub fn with_raw_chat_stream_strings(self, strings: Vec<String>) -> Self {
        *self.raw_chat_stream_strings.lock().unwrap() = strings;
        self
    }

    pub fn with_non_streaming_http_response(self, response: HttpResponse) -> Self {
        *self.non_streaming_http_response.lock().unwrap() = Some(response);
        self
    }
}

#[async_trait]
impl Transport for MockTransport {
    #[cfg_attr(feature = "tracing", instrument(skip(self, _request)))]
    async fn send_http_request(&self, _request: HttpRequest) -> Result<HttpResponse> {
        if let Some(response) = self.non_streaming_http_response.lock().unwrap().take() {
            Ok(response)
        } else {
            // Default empty response if no mock is configured
            Ok(HttpResponse { body: None })
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    async fn send_http_stream_request(
        &self,
        request: HttpRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        if request.url == "/api/chat" {
            let raw_responses = self
                .raw_chat_stream_strings
                .lock()
                .unwrap()
                .drain(..)
                .collect::<Vec<_>>();
            if !raw_responses.is_empty() {
                let byte_stream = stream::iter(raw_responses)
                    .map(|s| Ok(Bytes::from(format!("{}\n", s))))
                    .boxed();
                return Ok(byte_stream);
            }

            let chat_events = self
                .chat_stream_events
                .lock()
                .unwrap()
                .drain(..)
                .collect::<Vec<_>>();
            if !chat_events.is_empty() {
                let byte_stream = stream::iter(chat_events)
                    .map(|event| {
                        let json_string = serde_json::to_string(&event).map_err(|e| {
                            Error::Protocol(format!("Failed to serialize mock event: {}", e))
                        })?;
                        Ok(Bytes::from(format!("{}\n", json_string)))
                    })
                    .boxed();
                return Ok(byte_stream);
            }
        } else if request.url == "/api/generate" {
            let generate_bytes = self
                .generate_stream_bytes
                .lock()
                .unwrap()
                .drain(..)
                .collect::<Vec<_>>();
            if !generate_bytes.is_empty() {
                let byte_stream = stream::iter(generate_bytes).map(Ok).boxed();
                return Ok(byte_stream);
            }
        }

        // Default empty stream if no mock is configured for the given request
        Ok(stream::empty().boxed())
    }
}
