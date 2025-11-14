use std::sync::Arc;

use futures::StreamExt;

use ollama_sdk::transport::mock_transport::MockTransport;
use ollama_sdk::types::chat::{
    ChatRequestMessage, ChatResponse, ChatResponseMessage, ChatStreamEvent, SimpleChatRequest,
    StreamingChatRequest,
};
use ollama_sdk::types::Role;
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
    let mock_transport =
        Arc::new(MockTransport::new().with_non_streaming_response(expected_response.clone()));

    let client = OllamaClient::builder()
        .base_url("http://mock.ollama.ai")
        .transport(mock_transport) // Pass the mock transport to the builder
        .build()?;

    let request = SimpleChatRequest {
        model: "test-model".to_string(),
        messages: vec![ChatRequestMessage {
            role: Role::User,
            content: "Hi".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let response = client.chat_simple(request).await?;
    assert_eq!(response.message.content, expected_response.message.content);

    Ok(())
}

#[tokio::test]
async fn test_chat_stream() -> Result<()> {
    let mock_transport = Arc::new(MockTransport::new().with_streaming_raw_responses(vec![
        r#"{"model":"test-model","message":{"role":"assistant","content":"Hello"},"done":false}"#.to_string(),
        r#"{"model":"test-model","message":{"role":"assistant","content":" world"},"done":false}"#.to_string(),
        r#"{"model":"test-model","message":{"role":"assistant","content":"final message"},"done":true}"#.to_string(),
    ]));

    let client = OllamaClient::builder()
        .base_url("http://mock.ollama.ai")
        .transport(mock_transport)
        .build()?;

    let request = StreamingChatRequest {
        model: "test-model".to_string(),
        messages: vec![ChatRequestMessage {
            role: Role::User,
            content: "Stream me".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

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
