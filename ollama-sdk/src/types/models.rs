use ollama_sdk_macros::FromBytes;
use serde::{Deserialize, Serialize};

/// Represents the response from listing all available models on the Ollama server.
#[derive(Deserialize, Serialize, Default, FromBytes, Debug)]
pub struct ListModelsResponse {
    /// A list of available Ollama models.
    pub models: Vec<OllamaModel>,
}

/// Represents a single Ollama model available on the server.
#[derive(Deserialize, Default, Serialize, Debug)]
pub struct OllamaModel {
    /// The name of the model (e.g., "llama2").
    pub name: String,
    /// The timestamp when the model was last modified (ISO 8601 format).
    pub modified_at: String,
    /// The size of the model in bytes.
    pub size: u64,
    /// The digest of the model.
    pub digest: String,
    /// Detailed information about the model.
    pub details: OllamaModelDetails,
}

/// Provides detailed information about an Ollama model.
#[derive(Deserialize, Default, Serialize, Debug)]
pub struct OllamaModelDetails {
    /// The format of the model.
    pub format: String,
    /// The family of the model (e.g., "llama").
    pub family: String,
    /// A list of families the model belongs to.
    pub families: Vec<String>,
    /// The parameter size of the model (e.g., "7B").
    pub parameter_size: String,
    /// The quantization level of the model (e.g., "Q4_0").
    pub quantization_level: String,
}

/// Represents the response from listing models currently running on the Ollama server.
#[derive(Deserialize, Serialize, Default, FromBytes, Debug)]
pub struct ListRunningModelsResponse {
    /// A list of currently running Ollama models.
    pub models: Vec<OllamaRunningModel>,
}

/// Represents a single Ollama model that is currently running.
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct OllamaRunningModel {
    /// The name of the running model.
    pub model: String,
    /// The size of the model in bytes.
    pub size: u64,
    /// The digest of the model.
    pub digest: String,
    /// Detailed information about the running model.
    pub details: OllamaRunningModelDetails,
    /// The timestamp when the model is expected to expire (ISO 8601 format).
    pub expires_at: String,
    /// The VRAM usage of the model in bytes.
    pub size_vram: u64,
    /// The context length of the model.
    pub context_length: u32,
}

/// Provides detailed information about an Ollama model that is currently running.
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct OllamaRunningModelDetails {
    /// The parent model of the running model.
    pub parent_model: String,
    /// The format of the model.
    pub format: String,
    /// The family of the model.
    pub family: String,
    /// A list of families the model belongs to.
    pub families: Vec<String>,
    /// The parameter size of the model.
    pub parameter_size: String,
    /// The quantization level of the model.
    pub quantization_level: String,
}
