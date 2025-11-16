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

/// A mock implementation of the [`Transport`] trait for testing purposes.
///
/// This transport allows you to pre-configure responses for both streaming
/// and non-streaming HTTP requests, enabling isolated testing of client logic
/// without making actual network calls.
#[derive(Clone, Default)]
pub struct MockTransport {
    /// Stores a sequence of [`ChatStreamEvent`]s to be returned for streaming chat requests.
    chat_stream_events: Arc<Mutex<Vec<ChatStreamEvent>>>,
    /// Stores a sequence of raw `Bytes` to be returned for streaming generate requests.
    generate_stream_bytes: Arc<Mutex<Vec<Bytes>>>,
    /// Stores a sequence of raw JSON strings to be returned for streaming chat requests.
    raw_chat_stream_strings: Arc<Mutex<Vec<String>>>,

    /// Stores an optional [`HttpResponse`] to be returned for non-streaming HTTP requests.
    non_streaming_http_response: Arc<Mutex<Option<HttpResponse>>>,
}

impl MockTransport {
    /// Creates a new, empty [`MockTransport`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the mock to return a specific sequence of [`ChatStreamEvent`]s
    /// for streaming chat requests (`/api/chat`).
    pub fn with_chat_stream_events(self, events: Vec<ChatStreamEvent>) -> Self {
        *self.chat_stream_events.lock().unwrap() = events;
        self
    }

    /// Configures the mock to return a specific sequence of raw `Bytes`
    /// for streaming generate requests (`/api/generate`).
    pub fn with_generate_stream_bytes(self, bytes: Vec<Bytes>) -> Self {
        *self.generate_stream_bytes.lock().unwrap() = bytes;
        self
    }

    /// Configures the mock to return a specific sequence of raw JSON strings
    /// for streaming chat requests (`/api/chat`). Each string will be treated
    /// as a separate line in the stream.
    pub fn with_raw_chat_stream_strings(self, strings: Vec<String>) -> Self {
        *self.raw_chat_stream_strings.lock().unwrap() = strings;
        self
    }

    /// Configures the mock to return a specific [`HttpResponse`]
    /// for the next non-streaming HTTP request.
    pub fn with_non_streaming_http_response(self, response: HttpResponse) -> Self {
        *self.non_streaming_http_response.lock().unwrap() = Some(response);
        self
    }
}

#[async_trait]
impl Transport for MockTransport {
    /// Mocks sending a non-streaming HTTP request.
    ///
    /// If a `non_streaming_http_response` has been configured, it will be returned.
    /// Otherwise, an empty [`HttpResponse`] is returned.
    #[cfg_attr(feature = "tracing", instrument(skip(self, _request)))]
    async fn send_http_request(&self, _request: HttpRequest) -> Result<HttpResponse> {
        if let Some(response) = self.non_streaming_http_response.lock().unwrap().take() {
            Ok(response)
        } else {
            // Default empty response if no mock is configured
            Ok(HttpResponse { body: None })
        }
    }

    /// Mocks sending a streaming HTTP request.
    ///
    /// Depending on the request URL and configured mock data:
    /// - For `/api/chat`, it returns a stream of serialized [`ChatStreamEvent`]s
    ///   or raw JSON strings if `raw_chat_stream_strings` is set.
    /// - For `/api/generate`, it returns a stream of raw `Bytes` if `generate_stream_bytes` is set.
    /// If no relevant mock data is found, an empty stream is returned.
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
