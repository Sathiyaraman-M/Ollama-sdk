use std::pin::Pin;

use crate::types::Thinking;
use crate::Result;
use futures::Stream;
use ollama_sdk_macros::FromBytes;
use serde::{Deserialize, Serialize};

use super::{Role, ThinkingLevel};

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

impl ChatRequestMessage {
    pub fn new(role: Role, content: String) -> Self {
        Self {
            role,
            content,
            tool_calls: Vec::new(),
        }
    }

    pub fn set_tool_calls(mut self, tool_calls: Vec<FunctionalTool>) -> Self {
        self.tool_calls = tool_calls;
        self
    }
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

impl SimpleChatRequest {
    pub fn new(model: String, messages: Vec<ChatRequestMessage>) -> Self {
        Self {
            model,
            messages,
            think: Thinking::default(),
        }
    }

    pub fn enable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(true);
        self
    }

    pub fn disable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(false);
        self
    }

    pub fn set_thinking_level(mut self, level: ThinkingLevel) -> Self {
        self.think = Thinking::Level(level);
        self
    }
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct StreamingChatRequest {
    pub model: String,
    pub messages: Vec<ChatRequestMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    pub think: Thinking,
}

impl StreamingChatRequest {
    pub fn new(model: String, messages: Vec<ChatRequestMessage>) -> Self {
        Self {
            model,
            messages,
            tools: None,
            think: Thinking::default(),
        }
    }

    pub fn enable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(true);
        self
    }

    pub fn disable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(false);
        self
    }

    pub fn set_thinking_level(mut self, level: ThinkingLevel) -> Self {
        self.think = Thinking::Level(level);
        self
    }

    pub fn tools(mut self, tools: Vec<ToolSpec>) -> Self {
        self.tools = Some(tools);
        self
    }
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
