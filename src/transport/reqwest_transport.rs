use std::pin::Pin;

#[cfg(feature = "tracing")]
use tracing::instrument;

use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use reqwest::{Client, Url};

use crate::errors::{Error, Result};
use crate::transport::Transport;
use crate::types::ChatRequest;

pub struct ReqwestTransport {
    client: Client,
    base_url: Url,
    api_key: Option<String>,
}

impl ReqwestTransport {
    pub fn new(base_url: Url, api_key: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .build()
            .map_err(|e| Error::Client(e.to_string()))?;
        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }
}

#[async_trait]
impl Transport for ReqwestTransport {
    #[cfg_attr(feature = "tracing", instrument(skip(self, request)))]
    async fn send_chat_request(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<Bytes>> + Send>>> {
        let mut url = self
            .base_url
            .join("/api/chat")
            .map_err(|e| Error::Client(e.to_string()))?;
        if request.stream.unwrap_or(false) {
            url.query_pairs_mut().append_pair("stream", "true");
        }

        let mut request_builder = self.client.post(url).json(&request);

        if let Some(api_key) = &self.api_key {
            request_builder = request_builder.bearer_auth(api_key);
        }

        let response = request_builder.send().await.map_err(Error::Transport)?;

        response.error_for_status_ref().map_err(Error::Transport)?;

        let stream = response
            .bytes_stream()
            .map(|item| item.map_err(Error::Transport))
            .boxed();

        Ok(stream)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(self, result)))]
    async fn send_tool_result(&self, invocation_id: &str, result: serde_json::Value) -> Result<()> {
        let url = self
            .base_url
            .join(&format!("/v1/calls/{}/result", invocation_id))
            .map_err(|e| Error::Client(e.to_string()))?;

        let mut request_builder = self
            .client
            .post(url)
            .json(&serde_json::json!({ "result": result }));

        if let Some(api_key) = &self.api_key {
            request_builder = request_builder.bearer_auth(api_key);
        }

        let response = request_builder.send().await.map_err(Error::Transport)?;

        response.error_for_status().map_err(Error::Transport)?;

        Ok(())
    }
}
