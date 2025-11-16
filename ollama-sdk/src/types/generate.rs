use std::pin::Pin;

use crate::types::Thinking;
use crate::Result;
use futures::Stream;
use ollama_sdk_macros::FromBytes;
use serde::{Deserialize, Serialize};

use super::ThinkingLevel;

#[derive(Serialize, Default, Debug, Clone)]
pub struct GenerateRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<Thinking>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct GenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u16>,
}

#[derive(Deserialize, Serialize, Default, FromBytes, Debug, Clone)]
pub struct GenerateResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    #[serde(default)]
    pub thinking: String,
    pub done: bool,
    #[serde(default)]
    pub done_reason: Option<String>,
    #[serde(default)]
    pub total_duration: u64,
    #[serde(default)]
    pub load_duration: u64,
    #[serde(default)]
    pub prompt_eval_count: u64,
    #[serde(default)]
    pub prompt_eval_duration: u64,
    #[serde(default)]
    pub eval_count: u64,
    #[serde(default)]
    pub eval_duration: u64,
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct SimpleGenerateRequest {
    pub model: String,
    pub prompt: Option<String>,
    pub suffix: Option<String>,
    pub images: Option<Vec<String>>,
    pub system: Option<String>,
    pub think: Option<Thinking>,
    pub raw: Option<bool>,
    pub options: Option<GenerateOptions>,
}

impl SimpleGenerateRequest {
    pub fn new(model: String, prompt: String) -> Self {
        Self {
            model,
            prompt: Some(prompt),
            ..Default::default()
        }
    }

    pub fn enable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(true).into();
        self
    }

    pub fn disable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(false).into();
        self
    }

    pub fn set_thinking_level(mut self, level: ThinkingLevel) -> Self {
        self.think = Thinking::Level(level).into();
        self
    }

    pub fn system(mut self, system: String) -> Self {
        self.system = Some(system);
        self
    }

    pub fn images(mut self, images: Vec<String>) -> Self {
        self.images = Some(images);
        self
    }

    pub fn options(mut self, options: GenerateOptions) -> Self {
        self.options = Some(options);
        self
    }
}

impl From<SimpleGenerateRequest> for GenerateRequest {
    fn from(request: SimpleGenerateRequest) -> GenerateRequest {
        GenerateRequest {
            model: request.model,
            prompt: request.prompt,
            suffix: request.suffix,
            images: request.images,
            system: request.system,
            think: request.think,
            raw: request.raw,
            options: request.options,
            stream: false,
        }
    }
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct StreamingGenerateRequest {
    pub model: String,
    pub prompt: Option<String>,
    pub suffix: Option<String>,
    pub images: Option<Vec<String>>,
    pub system: Option<String>,
    pub think: Option<Thinking>,
    pub raw: Option<bool>,
    pub options: Option<GenerateOptions>,
}

impl StreamingGenerateRequest {
    pub fn new(model: String, prompt: String) -> Self {
        Self {
            model,
            prompt: Some(prompt),
            ..Default::default()
        }
    }

    pub fn enable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(true).into();
        self
    }

    pub fn disable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(false).into();
        self
    }

    pub fn set_thinking_level(mut self, level: ThinkingLevel) -> Self {
        self.think = Thinking::Level(level).into();
        self
    }

    pub fn system(mut self, system: String) -> Self {
        self.system = Some(system);
        self
    }

    pub fn images(mut self, images: Vec<String>) -> Self {
        self.images = Some(images);
        self
    }

    pub fn options(mut self, options: GenerateOptions) -> Self {
        self.options = Some(options);
        self
    }
}

impl From<StreamingGenerateRequest> for GenerateRequest {
    fn from(request: StreamingGenerateRequest) -> GenerateRequest {
        GenerateRequest {
            model: request.model,
            prompt: request.prompt,
            suffix: request.suffix,
            images: request.images,
            system: request.system,
            think: request.think,
            raw: request.raw,
            options: request.options,
            stream: true,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum GenerateStreamEvent {
    MessageChunk(GenerateResponse),
    Error(String),
    Partial {
        partial: String,
        error: Option<String>,
    },
}

// GenerateStream type
pub struct GenerateStream {
    pub inner: Pin<Box<dyn Stream<Item = Result<GenerateStreamEvent>> + Send>>,
}

impl Stream for GenerateStream {
    type Item = Result<GenerateStreamEvent>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}
