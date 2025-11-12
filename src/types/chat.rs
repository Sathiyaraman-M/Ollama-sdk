use crate::types::{Message, Thinking};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Default, Debug, Clone)]
pub struct SimpleChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub think: Thinking,
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct StreamingChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    pub think: Thinking,
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    #[serde(default)]
    pub think: Thinking,
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ToolSpec {
    #[serde(rename = "function")]
    Function(FunctionalTool),
}

#[derive(Serialize, Debug, Clone)]
pub struct FunctionalTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum StreamEvent {
    Partial {
        message: Message,
    },
    ToolCall {
        invocation_id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResultAck {
        invocation_id: String,
        name: String,
        result: serde_json::Value,
    },
    Metadata {
        info: serde_json::Value,
    },
    Done {
        final_message: Option<Message>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatResponse {
    pub message: Message,
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
