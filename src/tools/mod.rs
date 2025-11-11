use std::sync::Arc;

#[cfg(feature = "tracing")]
use tracing::instrument;

use async_trait::async_trait;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::errors::Error;

pub mod registry;

/// Context provided to a tool during its execution.
#[derive(Clone)]
pub struct ToolContext {
    /// A token to signal cancellation of the tool's execution.
    pub cancellation_token: CancellationToken,
    // Add other context information here, e.g., access to HTTP client, logger, etc.
}

#[async_trait]
pub trait Tool: Send + Sync + 'static {
    /// Returns the name of the tool. This name must be unique and match the name
    /// the model expects to call.
    fn name(&self) -> &str;

    /// Executes the tool with the given input.
    /// The input is typically a JSON object provided by the model.
    /// The tool should return a JSON object as its result.
    #[cfg_attr(feature = "tracing", instrument(skip(self, input, ctx)))]
    async fn call(&self, input: Value, ctx: ToolContext) -> Result<Value, Error>;
}

pub type DynTool = Arc<dyn Tool>;
