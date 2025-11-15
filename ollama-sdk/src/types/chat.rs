use std::pin::Pin;

use crate::types::Thinking;
use crate::Result;
use futures::Stream;
use ollama_sdk_macros::FromBytes;
use serde::{Deserialize, Serialize};

use super::Role;

#[derive(Serialize, Default, Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatRequestMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    #[serde(default)]
    pub think: Thinking,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ChatRequestMessage {
    pub role: Role,
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<FunctionalTool>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ToolSpec {
    #[serde(rename = "function")]
    Function(FunctionalTool),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FunctionalTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

#[derive(Deserialize, Serialize, Default, FromBytes, Debug, Clone)]
pub struct ChatResponse {
    pub model: String,
    #[serde(default)]
    pub created_at: String,
    pub message: ChatResponseMessage,
    pub done: bool,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ChatResponseMessage {
    pub role: Role,
    pub content: String,
    #[serde(default)]
    pub thinking: String,
    #[serde(default)]
    pub tool_calls: Vec<FunctionalTool>,
}

impl From<ChatResponseMessage> for ChatRequestMessage {
    fn from(value: ChatResponseMessage) -> Self {
        ChatRequestMessage {
            role: value.role,
            content: value.content,
            tool_calls: value.tool_calls,
        }
    }
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct SimpleChatRequest {
    pub model: String,
    pub messages: Vec<ChatRequestMessage>,
    pub think: Thinking,
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct StreamingChatRequest {
    pub model: String,
    pub messages: Vec<ChatRequestMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    pub think: Thinking,
}

impl From<SimpleChatRequest> for ChatRequest {
    fn from(value: SimpleChatRequest) -> Self {
        ChatRequest {
            model: value.model,
            messages: value.messages,
            stream: Some(false),
            think: value.think,
            tools: None,
        }
    }
}

impl From<StreamingChatRequest> for ChatRequest {
    fn from(value: StreamingChatRequest) -> Self {
        ChatRequest {
            model: value.model,
            messages: value.messages,
            stream: Some(true),
            think: value.think,
            tools: value.tools,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum ChatStreamEvent {
    Message(ChatResponse),
    Error(String),
    Partial {
        partial: String,
        error: Option<String>,
    },
}

// ChatStream type
pub struct ChatStream {
    pub inner: Pin<Box<dyn Stream<Item = Result<ChatStreamEvent>> + Send>>,
}

impl Stream for ChatStream {
    type Item = Result<ChatStreamEvent>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}
