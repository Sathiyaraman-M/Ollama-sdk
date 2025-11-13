use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::errors::Result;
use crate::types::chat::ChatStreamEvent;
use crate::types::{Message, Role};

pub struct ChatStreamParser<S>
where
    S: Stream<Item = Result<Bytes>> + Send + Unpin,
{
    inner: S,
    buffer: Vec<u8>,
}

impl<S> ChatStreamParser<S>
where
    S: Stream<Item = Result<Bytes>> + Send + Unpin,
{
    pub fn new(stream: S) -> Self {
        Self {
            inner: stream,
            buffer: Vec::new(),
        }
    }

    fn parse_lines(&mut self) -> Option<Result<ChatStreamEvent>> {
        loop {
            let newline_pos = self.buffer.iter().position(|&b| b == b'\n')?;
            let line_bytes = self.buffer.drain(..=newline_pos).collect::<Vec<u8>>(); // inclusive
            let line_str = String::from_utf8_lossy(&line_bytes);
            let line_str = line_str.trim();

            if line_str.is_empty() {
                continue; // Skip empty lines
            }

            match serde_json::from_str::<ChatStreamEvent>(line_str) {
                Ok(event) => return Some(Ok(event)),
                Err(_) => {
                    // If it's not a known StreamEvent, try to parse as a partial message
                    // This is a fallback for non-JSON fragments or unexpected formats.
                    // The technical design says: "Accept non-JSON fragments: attempt JSON parse, fallback to treating as Partial with content text."
                    // This means if it's not a valid StreamEvent JSON, we assume it's just raw text.
                    return Some(Ok(ChatStreamEvent::Partial {
                        message: Message {
                            role: Role::Assistant,
                            content: line_str.to_string(),
                        },
                    }));
                }
            }
        }
    }
}

impl<S> Stream for ChatStreamParser<S>
where
    S: Stream<Item = Result<Bytes>> + Send + Unpin,
{
    type Item = Result<ChatStreamEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // 1. Try to parse any complete lines in buffer
            if let Some(event) = self.parse_lines() {
                return Poll::Ready(Some(event));
            }

            // 2. If no complete line, check if stream is done
            if self.buffer.is_empty() {
                // Only poll inner if buffer is empty
                match Pin::new(&mut self.inner).poll_next(cx) {
                    Poll::Ready(Some(Ok(bytes))) => {
                        self.buffer.extend_from_slice(&bytes);
                        continue; // loop: try parse again
                    }
                    Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                    Poll::Ready(None) => return Poll::Ready(None), // stream ended, buffer empty
                    Poll::Pending => return Poll::Pending,
                }
            } else {
                // Buffer has data, but no newline → need more
                // Poll inner stream
                match Pin::new(&mut self.inner).poll_next(cx) {
                    Poll::Ready(Some(Ok(bytes))) => {
                        self.buffer.extend_from_slice(&bytes);
                        continue;
                    }
                    Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                    Poll::Ready(None) => {
                        // Stream ended with partial data
                        let content = String::from_utf8_lossy(&self.buffer).to_string();
                        self.buffer.clear();
                        if !content.trim().is_empty() {
                            return Poll::Ready(Some(Ok(ChatStreamEvent::Partial {
                                message: Message {
                                    role: Role::Assistant,
                                    content: content.trim().to_string(),
                                },
                            })));
                        }
                        return Poll::Ready(None);
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }
        }
    }
}
