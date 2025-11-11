use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;

use crate::errors::Result;
use crate::types::ChatRequest;

pub mod reqwest_transport;

#[async_trait]
pub trait Transport: Send + Sync + 'static {
    async fn send_chat_request(
        &self,
        request: ChatRequest,
    ) -> Result<Box<dyn Stream<Item = Result<Bytes>> + Send + Unpin>>;

    async fn send_tool_result(
        &self,
        invocation_id: &str,
        result: serde_json::Value,
    ) -> Result<()>;
}
