use std::pin::Pin;

#[cfg(feature = "tracing")]
use tracing::instrument;

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;

use crate::errors::Result;
use crate::types::ChatRequest;

pub mod mock_transport;
pub mod reqwest_transport;

#[async_trait]
pub trait Transport: Send + Sync + 'static {
    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    async fn send_chat_request(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>>;

    #[cfg_attr(feature = "tracing", instrument(skip(self, result)))]
    async fn send_tool_result(&self, invocation_id: &str, result: serde_json::Value) -> Result<()>;
}
