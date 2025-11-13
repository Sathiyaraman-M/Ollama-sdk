pub mod chat;
pub mod generate;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Thinking {
    Boolean(bool),
    Level(ThinkingLevel),
}

impl Default for Thinking {
    fn default() -> Self {
        Self::Boolean(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingLevel {
    High,
    Medium,
    Low,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OllamaError {
    pub error: String,
}
