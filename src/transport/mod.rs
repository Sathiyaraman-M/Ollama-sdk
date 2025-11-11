use std::pin::Pin;

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;

use crate::errors::Result;
use crate::types::ChatRequest;

pub mod mock_transport;
pub mod reqwest_transport;

#[async_trait]
pub trait Transport: Send + Sync + 'static {
    async fn send_chat_request(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>>;

    async fn send_tool_result(&self, invocation_id: &str, result: serde_json::Value) -> Result<()>;
}
