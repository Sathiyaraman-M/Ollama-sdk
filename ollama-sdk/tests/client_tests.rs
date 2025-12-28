use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;

use ollama_sdk::transport::MockTransport;
use ollama_sdk::types::chat::{
    ChatResponse, ChatResponseMessage, ChatStreamEvent, RegularChatRequestMessage,
    SimpleChatRequest, StreamingChatRequest,
};
use ollama_sdk::types::{HttpResponse, Role};
use ollama_sdk::OllamaClient;
use ollama_sdk::Result;

#[tokio::test]
async fn test_chat_simple() -> Result<()> {
    let expected_response = ChatResponse {
        message: ChatResponseMessage {
            role: Role::Assistant,
            content: "Hello from mock!".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let http_response_body = serde_json::to_vec(&expected_response)?; // Serialize ChatResponse to bytes
    let mock_transport = Arc::new(MockTransport::new().with_non_streaming_http_response(
        HttpResponse {
            body: Bytes::from(http_response_body).into(),
        },
    ));

    let client = OllamaClient::builder()
        .base_url("http://mock.ollama.ai")
        .transport(mock_transport) // Pass the mock transport to the builder
        .build()?;

    let request = SimpleChatRequest::new("test-model".to_string())
        .add_message(RegularChatRequestMessage::new(Role::User, "Hi".to_string()));

    let response = client.chat_simple(request).await?;
    assert_eq!(response.message.content, expected_response.message.content);

    Ok(())
}

#[tokio::test]
async fn test_chat_stream() -> Result<()> {
    let mock_transport = Arc::new(MockTransport::new().with_raw_chat_stream_strings(vec![
        r#"{"model":"test-model","message":{"role":"assistant","content":"Hello"},"done":false}"#.to_string(),
        r#"{"model":"test-model","message":{"role":"assistant","content":" world"},"done":false}"#.to_string(),
        r#"{"model":"test-model","message":{"role":"assistant","content":"final message"},"done":true}"#.to_string(),
    ]));

    let client = OllamaClient::builder()
        .base_url("http://mock.ollama.ai")
        .transport(mock_transport)
        .build()?;

    let request = StreamingChatRequest::new("test-model".to_string()).add_regular_message(
        RegularChatRequestMessage::new(Role::User, "Stream me".to_string()),
    );

    let mut stream = client.chat_stream(request).await?;
    let mut received_content = String::new();

    while let Some(event_res) = stream.next().await {
        match event_res? {
            ChatStreamEvent::Message(response) => {
                received_content.push_str(&response.message.content);
            }
            ChatStreamEvent::Error(error) => {
                received_content.push_str(format!("\nError: {}", error).as_str());
            }
            ChatStreamEvent::Partial { partial, error } => {
                received_content.push_str(
                    format!(
                        "\nUnknown Chunk: {}\nError Text: {}",
                        partial,
                        error.unwrap_or("Unknown".to_string())
                    )
                    .as_str(),
                );
            }
        }
    }

    assert_eq!(received_content, "Hello worldfinal message");
    Ok(())
}
