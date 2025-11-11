use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::errors::Result;
use crate::types::{Message, Role, StreamEvent};

pub struct StreamParser<S>
where
    S: Stream<Item = Result<Bytes>> + Send + Unpin,
{
    inner: S,
    buffer: Vec<u8>,
}

impl<S> StreamParser<S>
where
    S: Stream<Item = Result<Bytes>> + Send + Unpin,
{
    pub fn new(stream: S) -> Self {
        Self {
            inner: stream,
            buffer: Vec::new(),
        }
    }

    fn parse_lines(&mut self) -> Option<Result<StreamEvent>> {
        while let Some(newline_pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = self.buffer.drain(..(newline_pos + 1)).collect::<Vec<u8>>();
            let line_str = String::from_utf8_lossy(&line_bytes).trim().to_string();

            if line_str.is_empty() {
                continue; // Skip empty lines
            }

            match serde_json::from_str::<StreamEvent>(&line_str) {
                Ok(event) => return Some(Ok(event)),
                Err(_) => {
                    // If it's not a known StreamEvent, try to parse as a partial message
                    // This is a fallback for non-JSON fragments or unexpected formats.
                    // The technical design says: "Accept non-JSON fragments: attempt JSON parse, fallback to treating as Partial with content text."
                    // This means if it's not a valid StreamEvent JSON, we assume it's just raw text.
                    return Some(Ok(StreamEvent::Partial {
                        message: Message {
                            role: Role::Assistant,
                            content: line_str,
                            name: None,
                            metadata: None,
                        },
                    }));
                }
            }
        }
        None
    }
}

impl<S> Stream for StreamParser<S>
where
    S: Stream<Item = Result<Bytes>> + Send + Unpin,
{
    type Item = Result<StreamEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // First, try to parse any events already in the buffer
        if let Some(event) = self.parse_lines() {
            return Poll::Ready(Some(event));
        }

        // If buffer is empty or no complete event, poll the inner stream for more bytes
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                self.buffer.extend_from_slice(&bytes);
                // After extending, try parsing again
                if let Some(event) = self.parse_lines() {
                    Poll::Ready(Some(event))
                } else {
                    // If still no complete event, we need more data
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                // Inner stream has ended, try to parse any remaining data in the buffer
                if !self.buffer.is_empty() {
                    // Treat remaining buffer as a final partial message
                    let content = String::from_utf8_lossy(&self.buffer).to_string();
                    self.buffer.clear();
                    if !content.is_empty() {
                        return Poll::Ready(Some(Ok(StreamEvent::Partial {
                            message: Message {
                                role: Role::Assistant,
                                content,
                                name: None,
                                metadata: None,
                            },
                        })));
                    }
                }
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
