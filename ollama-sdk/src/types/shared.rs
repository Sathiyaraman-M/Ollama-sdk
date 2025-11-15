use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    #[default]
    User,
    Assistant,
    Tool,
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
