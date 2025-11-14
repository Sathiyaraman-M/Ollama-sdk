use std::pin::Pin;

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;

use crate::types::chat::ChatRequest;
use crate::types::generate::GenerateRequest;
use crate::Result;

pub mod mock_transport;
pub mod reqwest_transport;

#[async_trait]
pub trait Transport: Send + Sync + 'static {
    async fn send_generate_request(
        &self,
        request: GenerateRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>>;

    async fn send_chat_request(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>>;
}
