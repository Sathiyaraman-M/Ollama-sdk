//! Provides a generic parser for parsing and handling streaming responses
//! from the Ollama API.
//!
//! This module contains parsers for different types of streaming events,
//! such as chat completions and text generation.

use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::types::OllamaError;
use crate::Result;
use bytes::Bytes;
use futures::Stream;
use serde::de::DeserializeOwned;

/// Small conversion trait so endpoint-specific event enums can be constructed
/// from a successful message `M`, an error string, or a partial payload.
pub trait StreamEventExt<M>: Sized {
    /// Create an event from a successfully deserialized message.
    fn from_message(msg: M) -> Self;

    /// Create an event from an error string.
    fn from_error(err: String) -> Self;

    /// Create a partial event (with optional error text).
    fn partial(partial: String, error: Option<String>) -> Self;
}

/// Generic newline-delimited JSON streaming parser.
///
/// - `S` is the underlying stream that yields `Result<Bytes>`
/// - `M` is the concrete message struct you expect per line (DeserializeOwned)
/// - `E` is the endpoint event enum type that implements `StreamEventExt<M>`
pub struct GenericStreamParser<S, M, E>
where
    S: Stream<Item = Result<Bytes>> + Send + Unpin,
    M: DeserializeOwned,
    E: StreamEventExt<M>,
{
    inner: S,
    buffer: Vec<u8>,
    _marker: PhantomData<(M, E)>,
}

impl<S, M, E> GenericStreamParser<S, M, E>
where
    S: Stream<Item = Result<Bytes>> + Send + Unpin,
    M: DeserializeOwned,
    E: StreamEventExt<M>,
{
    pub fn new(stream: S) -> Self {
        Self {
            inner: stream,
            buffer: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Try to parse one complete newline-terminated line from the buffer.
    /// Returns `Some(Ok(E))` when we parsed one event; `Some(Err(e))` for a transport/error;
    /// `None` when no full line is available yet.
    fn parse_lines(&mut self) -> Option<Result<E>> {
        loop {
            // find newline
            let newline_pos = self.buffer.iter().position(|&b| b == b'\n')?;
            // take inclusive newline bytes
            let line_bytes = self.buffer.drain(..=newline_pos).collect::<Vec<u8>>();
            let line_str = String::from_utf8_lossy(&line_bytes);
            let line_str = line_str.trim();

            if line_str.is_empty() {
                continue; // skip blank lines
            }

            // First try to parse the expected message type M
            match serde_json::from_str::<M>(line_str) {
                Ok(msg) => return Some(Ok(E::from_message(msg))),
                Err(e_msg) => {
                    // If not M, try to parse an OllamaError
                    match serde_json::from_str::<OllamaError>(line_str) {
                        Ok(err) => return Some(Ok(E::from_error(err.error))),
                        Err(_) => {
                            // fallback: treat as partial with parse error string
                            return Some(Ok(E::partial(
                                line_str.to_string(),
                                Some(e_msg.to_string()),
                            )));
                        }
                    }
                }
            }
        }
    }
}

impl<S, M, E> Stream for GenericStreamParser<S, M, E>
where
    S: Stream<Item = Result<Bytes>> + Send + Unpin,
    M: DeserializeOwned + Unpin,
    E: StreamEventExt<M> + Unpin,
{
    type Item = Result<E>;

    // remove `mut` from the `self` binding; we'll call `get_mut()` to get &mut Self.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // SAFETY / rationale:
        // We have exclusive access to the pinned reference in this poll method,
        // so calling `get_mut()` to obtain `&mut Self` for internal field mutation
        // is correct here.
        let this = self.get_mut();

        loop {
            // 1. Try to parse any complete lines in buffer
            if let Some(event) = this.parse_lines() {
                return Poll::Ready(Some(event));
            }

            // 2. If no complete line, check if stream is done
            if this.buffer.is_empty() {
                // Only poll inner if buffer is empty
                match Pin::new(&mut this.inner).poll_next(cx) {
                    Poll::Ready(Some(Ok(bytes))) => {
                        this.buffer.extend_from_slice(&bytes);
                        continue; // loop: try parse again
                    }
                    Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                    Poll::Ready(None) => return Poll::Ready(None), // stream ended, buffer empty
                    Poll::Pending => return Poll::Pending,
                }
            } else {
                // Buffer has data, but no newline → need more
                // Poll inner stream
                match Pin::new(&mut this.inner).poll_next(cx) {
                    Poll::Ready(Some(Ok(bytes))) => {
                        this.buffer.extend_from_slice(&bytes);
                        continue;
                    }
                    Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                    Poll::Ready(None) => {
                        // Stream ended with partial data
                        let content = String::from_utf8_lossy(&this.buffer).to_string();
                        this.buffer.clear();
                        if !content.trim().is_empty() {
                            return Poll::Ready(Some(Ok(E::partial(content, None))));
                        }
                        return Poll::Ready(None);
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }
        }
    }
}
