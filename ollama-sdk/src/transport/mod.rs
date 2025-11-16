use std::pin::Pin;

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;

use crate::types::{HttpRequest, HttpResponse};
use crate::Result;

mod mock_transport;
mod reqwest_transport;

pub use mock_transport::MockTransport;
pub use reqwest_transport::ReqwestTransport;

#[async_trait]
pub trait Transport: Send + Sync + 'static {
    /// Sends a non-streaming HTTP request and returns the response.
    async fn send_http_request(&self, request: HttpRequest) -> Result<HttpResponse>;

    /// Sends a streaming HTTP request and returns a stream of response bytes.
    async fn send_http_stream_request(
        &self,
        request: HttpRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>>;
}
