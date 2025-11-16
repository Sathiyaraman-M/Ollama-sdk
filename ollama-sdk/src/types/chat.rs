//! Contains all data structures that are particularly used for Ollama Chat API

use std::pin::Pin;

use crate::types::Thinking;
use crate::Result;
use futures::Stream;
use ollama_sdk_macros::FromBytes;
use serde::{Deserialize, Serialize};

use super::{Role, ThinkingLevel};

/// Represents a chat request to the Ollama API.
///
/// This struct is used to send a series of messages to a specified model
/// and receive a chat completion. It supports both streaming and non-streaming
/// responses, as well as tool integration.
#[derive(Serialize, Default, Debug, Clone)]
pub struct ChatRequest {
    /// The name of the model to use for the chat completion (e.g., "llama2").
    pub model: String,
    /// A list of messages exchanged in the chat.
    pub messages: Vec<ChatRequestMessage>,
    /// If `true`, the response will be streamed back as a series of [`ChatStreamEvent`]s.
    /// If `false` or `None`, a single [`ChatResponse`] will be returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// An optional list of tools that the model can use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    /// Configuration for the model's "thinking" process.
    #[serde(default)]
    pub think: Thinking,
}

/// Represents a single message in a chat request.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ChatRequestMessage {
    /// The role of the sender (e.g., `User`, `Assistant`, `System`).
    pub role: Role,
    /// The content of the message.
    pub content: String,
    /// An optional list of tool calls made by the assistant.
    #[serde(default)]
    pub tool_calls: Vec<FunctionalTool>,
}

impl ChatRequestMessage {
    /// Creates a new [`ChatRequestMessage`] with given [`Role`] and `content`
    pub fn new(role: Role, content: String) -> Self {
        Self {
            role,
            content,
            tool_calls: Vec::new(),
        }
    }

    /// Set a list of tool calls made by the LLM and returns the `ChatRequestMessage`
    pub fn set_tool_calls(mut self, tool_calls: Vec<FunctionalTool>) -> Self {
        self.tool_calls = tool_calls;
        self
    }
}

/// Specifies a tool that the model can use.
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ToolSpec {
    /// A functional tool definition.
    #[serde(rename = "function")]
    Function(FunctionalTool),
}

/// Represents a functional tool that the model can call.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FunctionalTool {
    /// The name of the tool.
    pub name: String,
    /// An optional description of the tool's purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The JSON schema for the tool's parameters.
    pub parameters: serde_json::Value,
}

/// Represents a chat response from the Ollama API.
///
/// This struct is used for non-streaming chat completions.
#[derive(Deserialize, Serialize, Default, FromBytes, Debug, Clone)]
pub struct ChatResponse {
    /// The name of the model that generated the response.
    pub model: String,
    /// The timestamp when the response was created.
    #[serde(default)]
    pub created_at: String,
    /// The message content from the model.
    pub message: ChatResponseMessage,
    /// Indicates if the chat completion is complete.
    pub done: bool,
}

/// Represents a single message in a chat response.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ChatResponseMessage {
    /// The role of the sender (e.g., `User`, `Assistant`).
    pub role: Role,
    /// The content of the message.
    pub content: String,
    /// The model's internal "thinking" process, if enabled.
    #[serde(default)]
    pub thinking: String,
    /// An optional list of tool calls made by the assistant.
    #[serde(default)]
    pub tool_calls: Vec<FunctionalTool>,
}

impl From<ChatResponseMessage> for ChatRequestMessage {
    /// Converts a [`ChatResponseMessage`] into a [`ChatRequestMessage`].
    /// This is useful for continuing a conversation where the model's response
    /// becomes part of the next request's message history.
    fn from(value: ChatResponseMessage) -> Self {
        ChatRequestMessage {
            role: value.role,
            content: value.content,
            tool_calls: value.tool_calls,
        }
    }
}

/// A simplified chat request for non-streaming responses.
///
/// This struct is a convenience wrapper for creating a [`ChatRequest`]
/// that explicitly requests a non-streaming response.
#[derive(Serialize, Default, Debug, Clone)]
pub struct SimpleChatRequest {
    /// The name of the model to use.
    pub model: String,
    /// The messages in the chat history.
    pub messages: Vec<ChatRequestMessage>,
    /// Configuration for the model's "thinking" process.
    pub think: Thinking,
}

impl SimpleChatRequest {
    /// Creates a new [`SimpleChatRequest`].
    pub fn new(model: String, messages: Vec<ChatRequestMessage>) -> Self {
        Self {
            model,
            messages,
            think: Thinking::default(),
        }
    }

    /// Sets `think` param in the API call to `true`.
    pub fn enable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(true);
        self
    }

    /// Sets `think` param in the API call to `false`.
    pub fn disable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(false);
        self
    }

    /// Sets `think` param in the API call to specified level (`high`, `medium`, `low`).
    pub fn set_thinking_level(mut self, level: ThinkingLevel) -> Self {
        self.think = Thinking::Level(level);
        self
    }
}

/// A simplified chat request for streaming responses.
///
/// This struct is a convenience wrapper for creating a [`ChatRequest`]
/// that explicitly requests a streaming response.
#[derive(Serialize, Default, Debug, Clone)]
pub struct StreamingChatRequest {
    /// The name of the model to use.
    pub model: String,
    /// The messages in the chat history.
    pub messages: Vec<ChatRequestMessage>,
    /// An optional list of tools that the model can use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    /// Configuration for the model's "thinking" process.
    pub think: Thinking,
}

impl StreamingChatRequest {
    /// Creates a new [`StreamingChatRequest`].
    pub fn new(model: String, messages: Vec<ChatRequestMessage>) -> Self {
        Self {
            model,
            messages,
            tools: None,
            think: Thinking::default(),
        }
    }

    /// Sets `think` param in the API call to `true`.
    pub fn enable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(true);
        self
    }

    /// Sets `think` param in the API call to `false`.
    pub fn disable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(false);
        self
    }

    /// Sets `think` param in the API call to specified level (`high`, `medium`, `low`).
    pub fn set_thinking_level(mut self, level: ThinkingLevel) -> Self {
        self.think = Thinking::Level(level);
        self
    }

    /// Sets the tools for the streaming chat request.
    pub fn tools(mut self, tools: Vec<ToolSpec>) -> Self {
        self.tools = Some(tools);
        self
    }
}

impl From<SimpleChatRequest> for ChatRequest {
    /// Converts a [`SimpleChatRequest`] into a [`ChatRequest`] with `stream` set to `false`.
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
    /// Converts a [`StreamingChatRequest`] into a [`ChatRequest`] with `stream` set to `true`.
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

/// Represents an event received from a streaming chat response.
#[derive(Deserialize, Serialize, Debug)]
pub enum ChatStreamEvent {
    /// A complete chat response message.
    Message(ChatResponse),
    /// An error occurred during the streaming process.
    Error(String),
    /// A partial response, returned when the content was un-parseable
    Partial {
        /// The un-parseable content.
        partial: String,
        /// An optional error message associated with the partial response.
        error: Option<String>,
    },
}

/// A stream of [`ChatStreamEvent`]s for streaming chat completions.
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
