//! Provides traits and types for defining and managing tools that can be used by Ollama models.
//!
//! This module enables the integration of external functionalities (tools) that models
//! can call to perform actions or retrieve information.

use std::sync::Arc;

#[cfg(feature = "tracing")]
use tracing::instrument;

use async_trait::async_trait;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::Error;

mod registry;

pub use registry::ToolRegistry;

/// Context provided to a tool during its execution.
///
/// This struct can hold various pieces of information that a tool might need
/// to perform its operation, such as cancellation tokens, access to external services,
/// or logging facilities.
#[derive(Clone)]
pub struct ToolContext {
    /// A token to signal cancellation of the tool's execution.
    /// Tools should periodically check this token and gracefully exit if cancellation is requested.
    pub cancellation_token: CancellationToken,
    // Add other context information here, e.g., access to HTTP client, logger, etc.
}

/// A trait for defining a tool that can be executed by an Ollama model.
///
/// Implementations of this trait define the tool's name and its execution logic.
#[async_trait]
pub trait Tool: Send + Sync + 'static {
    /// Returns the name of the tool.
    ///
    /// This name must be unique across all registered tools and should match the name
    /// the model expects to call.
    fn name(&self) -> &str;

    /// Executes the tool with the given input.
    ///
    /// The `input` is typically a JSON object provided by the model, containing
    /// the arguments for the tool. The tool should return a JSON `Value` as its result.
    ///
    /// # Arguments
    ///
    /// * `input` - A `serde_json::Value` representing the input arguments for the tool.
    /// * `ctx` - A [`ToolContext`] providing additional context for the tool's execution.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a `serde_json::Value` representing the tool's output,
    /// or an [`Error`](enum@crate::Error) if the tool execution fails.
    #[cfg_attr(feature = "tracing", instrument(skip(self, _input, _ctx)))]
    async fn call(&self, _input: Value, _ctx: ToolContext) -> Result<Value, Error> {
        Ok(Value::Null)
    }
}

/// A type alias for a dynamically dispatched [`Tool`] trait object.
///
/// This allows for storing and managing different tool implementations in a collection.
pub type DynTool = Arc<dyn Tool>;
