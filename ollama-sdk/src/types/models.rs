use ollama_sdk_macros::FromBytes;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, FromBytes, Debug)]
pub struct ListModelsResponse {
    pub models: Vec<OllamaModel>,
}

#[derive(Deserialize, Default, Serialize, Debug)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String, // ISO 8601 timestamp
    pub size: u64,
    pub digest: String,
    pub details: OllamaModelDetails,
}

#[derive(Deserialize, Default, Serialize, Debug)]
pub struct OllamaModelDetails {
    pub format: String,
    pub family: String,
    pub families: Vec<String>,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Deserialize, Serialize, Default, FromBytes, Debug)]
pub struct ListRunningModelsResponse {
    pub models: Vec<OllamaRunningModel>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct OllamaRunningModel {
    pub model: String,
    pub size: u64,
    pub digest: String,
    pub details: OllamaRunningModelDetails,
    pub expires_at: String, // ISO 8601 timestamp
    pub size_vram: u64,
    pub context_length: u32,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct OllamaRunningModelDetails {
    pub parent_model: String,
    pub format: String,
    pub family: String,
    pub families: Vec<String>,
    pub parameter_size: String,
    pub quantization_level: String,
}
