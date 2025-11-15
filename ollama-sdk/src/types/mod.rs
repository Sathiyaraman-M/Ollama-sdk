pub mod chat;
pub mod generate;
pub mod models;

use super::Result;

use bytes::Bytes;
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

#[derive(Default, Debug)]
pub struct HttpRequest {
    pub url: String,
    pub verb: HttpVerb,
    pub body: Option<serde_json::Value>,
}

#[derive(Default, Debug)]
pub enum HttpVerb {
    #[default]
    GET,
    POST,
    PUT,
    DELETE,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub body: Option<Bytes>,
}

impl HttpRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    pub fn get(mut self) -> Self {
        self.verb = HttpVerb::GET;
        self
    }

    pub fn post(mut self) -> Self {
        self.verb = HttpVerb::POST;
        self
    }

    pub fn put(mut self) -> Self {
        self.verb = HttpVerb::PUT;
        self
    }

    pub fn delete(mut self) -> Self {
        self.verb = HttpVerb::DELETE;
        self
    }

    pub fn body<T: Serialize>(mut self, body: T) -> Result<Self> {
        self.body = Some(serde_json::to_value(body)?);
        Ok(self)
    }
}
