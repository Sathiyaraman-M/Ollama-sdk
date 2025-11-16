//! Contains all data structures that are particularly used for Ollama Generate API

use std::pin::Pin;

use crate::types::Thinking;
use crate::Result;
use futures::Stream;
use ollama_sdk_macros::FromBytes;
use serde::{Deserialize, Serialize};

use super::ThinkingLevel;

/// Represents a request to the Ollama API for text generation.
///
/// This struct allows specifying the model, prompt, system message,
/// and various generation options. It supports both streaming and non-streaming
/// responses.
#[derive(Serialize, Default, Debug, Clone)]
pub struct GenerateRequest {
    /// The name of the model to use for generation (e.g., "llama2").
    pub model: String,
    /// The primary prompt for the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// A suffix to be appended to the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// Optional base64-encoded images to include in the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    /// A system message to guide the model's behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// If `true`, the response will be streamed back as a series of [`GenerateStreamEvent`]s.
    /// If `false`, a single [`GenerateResponse`] will be returned.
    pub stream: bool,
    /// Configuration for the model's "thinking" process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<Thinking>,
    /// If `true`, the raw prompt will be used without any templating.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<bool>,
    /// Additional generation options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
}

/// Represents various options that can be configured for text generation.
#[derive(Serialize, Default, Debug, Clone)]
pub struct GenerateOptions {
    /// The random seed for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u8>,
    /// The temperature for sampling, controlling randomness. Higher values mean more random.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// The top-k sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u8>,
    /// The top-p sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// The minimum-p sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f32>,
    /// A list of strings that, if generated, will stop the generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// The size of the context window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u16>,
    /// The maximum number of tokens to predict.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u16>,
}

/// Represents a response from the Ollama API for text generation.
///
/// This struct is used for non-streaming generation responses.
#[derive(Deserialize, Serialize, Default, FromBytes, Debug, Clone)]
pub struct GenerateResponse {
    /// The name of the model that generated the response.
    pub model: String,
    /// The timestamp when the response was created.
    pub created_at: String,
    /// The generated text response.
    pub response: String,
    /// The model's internal "thinking" process, if enabled.
    #[serde(default)]
    pub thinking: String,
    /// Indicates if the generation is complete.
    pub done: bool,
    /// The reason why the generation finished (e.g., "stop", "length").
    #[serde(default)]
    pub done_reason: Option<String>,
    /// The total duration of the generation process in nanoseconds.
    #[serde(default)]
    pub total_duration: u64,
    /// The duration spent loading the model in nanoseconds.
    #[serde(default)]
    pub load_duration: u64,
    /// The number of tokens in the prompt that were evaluated.
    #[serde(default)]
    pub prompt_eval_count: u64,
    /// The duration spent evaluating the prompt in nanoseconds.
    #[serde(default)]
    pub prompt_eval_duration: u64,
    /// The number of tokens generated.
    #[serde(default)]
    pub eval_count: u64,
    /// The duration spent generating tokens in nanoseconds.
    #[serde(default)]
    pub eval_duration: u64,
}

/// A simplified generation request for non-streaming responses.
///
/// This struct is a convenience wrapper for creating a [`GenerateRequest`]
/// that explicitly requests a non-streaming response.
#[derive(Serialize, Default, Debug, Clone)]
pub struct SimpleGenerateRequest {
    /// The name of the model to use.
    pub model: String,
    /// The primary prompt for the model.
    pub prompt: Option<String>,
    /// A suffix to be appended to the prompt.
    pub suffix: Option<String>,
    /// Optional base64-encoded images to include in the prompt.
    pub images: Option<Vec<String>>,
    /// A system message to guide the model's behavior.
    pub system: Option<String>,
    /// Configuration for the model's "thinking" process.
    pub think: Option<Thinking>,
    /// If `true`, the raw prompt will be used without any templating.
    pub raw: Option<bool>,
    /// Additional generation options.
    pub options: Option<GenerateOptions>,
}

impl SimpleGenerateRequest {
    /// Creates a new [`SimpleGenerateRequest`].
    pub fn new(model: String, prompt: String) -> Self {
        Self {
            model,
            prompt: Some(prompt),
            ..Default::default()
        }
    }

    /// Sets `think` param in the API call to `true`.
    pub fn enable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(true).into();
        self
    }

    /// Sets `think` param in the API call to `false`.
    pub fn disable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(false).into();
        self
    }

    /// Sets `think` param in the API call to specified level (`high`, `medium`, `low`).
    pub fn set_thinking_level(mut self, level: ThinkingLevel) -> Self {
        self.think = Thinking::Level(level).into();
        self
    }

    /// Sets the system message for the request.
    pub fn system(mut self, system: String) -> Self {
        self.system = Some(system);
        self
    }

    /// Sets the images for the request. An image should be a Base64-encoded string
    pub fn images(mut self, images: Vec<String>) -> Self {
        self.images = Some(images);
        self
    }

    /// Sets the generation options for the request.
    pub fn options(mut self, options: GenerateOptions) -> Self {
        self.options = Some(options);
        self
    }
}

impl From<SimpleGenerateRequest> for GenerateRequest {
    /// Converts a [`SimpleGenerateRequest`] into a [`GenerateRequest`] with `stream` set to `false`.
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

/// A simplified generation request for streaming responses.
///
/// This struct is a convenience wrapper for creating a [`GenerateRequest`]
/// that explicitly requests a streaming response.
#[derive(Serialize, Default, Debug, Clone)]
pub struct StreamingGenerateRequest {
    /// The name of the model to use.
    pub model: String,
    /// The primary prompt for the model.
    pub prompt: Option<String>,
    /// A suffix to be appended to the prompt.
    pub suffix: Option<String>,
    /// Optional base64-encoded images to include in the prompt.
    pub images: Option<Vec<String>>,
    /// A system message to guide the model's behavior.
    pub system: Option<String>,
    /// Configuration for the model's "thinking" process.
    pub think: Option<Thinking>,
    /// If `true`, the raw prompt will be used without any templating.
    pub raw: Option<bool>,
    /// Additional generation options.
    pub options: Option<GenerateOptions>,
}

impl StreamingGenerateRequest {
    /// Creates a new [`StreamingGenerateRequest`].
    pub fn new(model: String, prompt: String) -> Self {
        Self {
            model,
            prompt: Some(prompt),
            ..Default::default()
        }
    }

    /// Sets `think` param in the API call to `true`.
    pub fn enable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(true).into();
        self
    }

    /// Sets `think` param in the API call to `false`.
    pub fn disable_thinking(mut self) -> Self {
        self.think = Thinking::Boolean(false).into();
        self
    }

    /// Sets `think` param in the API call to specified level (`high`, `medium`, `low`).
    pub fn set_thinking_level(mut self, level: ThinkingLevel) -> Self {
        self.think = Thinking::Level(level).into();
        self
    }

    /// Sets the system message for the request.
    pub fn system(mut self, system: String) -> Self {
        self.system = Some(system);
        self
    }

    /// Sets the images for the request.
    pub fn images(mut self, images: Vec<String>) -> Self {
        self.images = Some(images);
        self
    }

    /// Sets the generation options for the request.
    pub fn options(mut self, options: GenerateOptions) -> Self {
        self.options = Some(options);
        self
    }
}

impl From<StreamingGenerateRequest> for GenerateRequest {
    /// Converts a [`StreamingGenerateRequest`] into a [`GenerateRequest`] with `stream` set to `true`.
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

/// Represents an event received from a streaming generation response.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum GenerateStreamEvent {
    /// A chunk of the generated response.
    MessageChunk(GenerateResponse),
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

/// A stream of [`GenerateStreamEvent`]s for streaming text generation.
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
