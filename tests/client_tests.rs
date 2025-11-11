use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use serde_json::json;

use ollama_sdk::client::OllamaClient;
use ollama_sdk::errors::{Error, Result};
use ollama_sdk::tools::{Tool, ToolContext};
use ollama_sdk::transport::mock_transport::MockTransport;
use ollama_sdk::types::{ChatRequest, ChatResponse, Message, Role, StreamEvent};

// --- Mock Tool Implementation ---
struct MockSearchTool;

#[async_trait::async_trait]
impl Tool for MockSearchTool {
    fn name(&self) -> &str {
        "search"
    }

    async fn call(&self, input: serde_json::Value, _ctx: ToolContext) -> Result<serde_json::Value> {
        if input["query"] == "rust" {
            Ok(json!({"result": "Rust programming language documentation"}))
        } else {
            Ok(json!({"result": "No relevant search results"}))
        }
    }
}

struct MockFailingTool;

#[async_trait::async_trait]
impl Tool for MockFailingTool {
    fn name(&self) -> &str {
        "failing_tool"
    }

    async fn call(
        &self,
        _input: serde_json::Value,
        _ctx: ToolContext,
    ) -> Result<serde_json::Value> {
        Err(Error::Tool("Tool failed intentionally".to_string()))
    }
}

// --- Tests ---

#[tokio::test]
async fn test_chat_non_streaming() -> Result<()> {
    let expected_response = ChatResponse {
        message: Message {
            role: Role::Assistant,
            content: "Hello from mock!".to_string(),
            name: None,
            metadata: None,
        },
    };
    let mock_transport =
        Arc::new(MockTransport::new().with_non_streaming_response(expected_response.clone()));

    let client = OllamaClient::builder()
        .base_url("http://mock.ollama.ai")
        .max_tool_runtime(Duration::from_secs(1))
        .transport(mock_transport.clone()) // Pass the mock transport to the builder
        .build()?;

    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "Hi".to_string(),
            name: None,
            metadata: None,
        }],
        stream: Some(false),
        max_tokens: None,
        temperature: None,
        tools: None,
        request_id: None,
    };

    let response = client.chat(request).await?;
    assert_eq!(response.message.content, expected_response.message.content);

    Ok(())
}

#[tokio::test]
async fn test_chat_stream_partial_events() -> Result<()> {
    let mock_transport = Arc::new(MockTransport::new().with_chat_responses(vec![
        StreamEvent::Partial {
            message: Message {
                role: Role::Assistant,
                content: "Hello".to_string(),
                name: None,
                metadata: None,
            },
        },
        StreamEvent::Partial {
            message: Message {
                role: Role::Assistant,
                content: " world".to_string(),
                name: None,
                metadata: None,
            },
        },
        StreamEvent::Done {
            final_message: None,
        },
    ]));

    let client = OllamaClient::builder()
        .base_url("http://mock.ollama.ai")
        .max_tool_runtime(Duration::from_secs(1))
        .transport(mock_transport.clone())
        .build()?;

    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "Stream me".to_string(),
            name: None,
            metadata: None,
        }],
        stream: Some(true),
        max_tokens: None,
        temperature: None,
        tools: None,
        request_id: None,
    };

    let mut stream = client.chat_stream(request).await?;
    let mut received_content = String::new();

    while let Some(event_res) = stream.next().await {
        let event = event_res?;
        if let StreamEvent::Partial { message } = event {
            received_content.push_str(&message.content);
        }
    }

    assert_eq!(received_content, "Hello world");
    Ok(())
}

#[tokio::test]
async fn test_chat_stream_tool_dispatch_success() -> Result<()> {
    let mock_transport = Arc::new(MockTransport::new().with_chat_responses(vec![
        StreamEvent::ToolCall {
            invocation_id: "inv-1".to_string(),
            name: "search".to_string(),
            input: json!({"query": "rust"}),
        },
        StreamEvent::Partial {
            message: Message {
                role: Role::Assistant,
                content: "Searching...".to_string(),
                name: None,
                metadata: None,
            },
        },
        StreamEvent::Done {
            final_message: None,
        },
    ]));

    let mut client = OllamaClient::builder()
        .base_url("http://mock.ollama.ai")
        .max_tool_runtime(Duration::from_secs(1))
        .transport(mock_transport.clone())
        .build()?;

    client.register_tool(Arc::new(MockSearchTool))?;

    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "Find rust docs".to_string(),
            name: None,
            metadata: None,
        }],
        stream: Some(true),
        max_tokens: None,
        temperature: None,
        tools: None,
        request_id: None,
    };

    let mut stream = client.chat_stream(request).await?;
    let mut received_content = String::new();
    let mut tool_call_received = false;

    while let Some(event_res) = stream.next().await {
        let event = event_res?;
        match event {
            StreamEvent::ToolCall { .. } => {
                tool_call_received = true;
            }
            StreamEvent::Partial { message } => {
                received_content.push_str(&message.content);
            }
            _ => {}
        }
    }

    assert!(tool_call_received);
    assert_eq!(received_content, "Searching...");

    // Give spawned task time to complete
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify that the tool result was sent back
    let tool_results = mock_transport.get_tool_results_sent();
    assert_eq!(tool_results.len(), 1);
    assert_eq!(tool_results[0].0, "inv-1");
    assert_eq!(
        tool_results[0].1,
        json!({"result": "Rust programming language documentation"})
    );

    Ok(())
}

#[tokio::test]
async fn test_chat_stream_tool_dispatch_failure() -> Result<()> {
    let mock_transport = Arc::new(MockTransport::new().with_chat_responses(vec![
        StreamEvent::ToolCall {
            invocation_id: "inv-2".to_string(),
            name: "failing_tool".to_string(),
            input: json!({}),
        },
        StreamEvent::Partial {
            message: Message {
                role: Role::Assistant,
                content: "Tool failed...".to_string(),
                name: None,
                metadata: None,
            },
        },
        StreamEvent::Done {
            final_message: None,
        },
    ]));

    let mut client = OllamaClient::builder()
        .base_url("http://mock.ollama.ai")
        .max_tool_runtime(Duration::from_secs(1))
        .transport(mock_transport.clone())
        .build()?;

    client.register_tool(Arc::new(MockFailingTool))?;

    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "Run failing tool".to_string(),
            name: None,
            metadata: None,
        }],
        stream: Some(true),
        max_tokens: None,
        temperature: None,
        tools: None,
        request_id: None,
    };

    let mut stream = client.chat_stream(request).await?;
    let mut tool_call_received = false;

    while let Some(event_res) = stream.next().await {
        let event = event_res?;
        if let StreamEvent::ToolCall { .. } = event {
            tool_call_received = true;
        }
    }

    assert!(tool_call_received);

    // Give spawned task time to complete
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify that the tool result was sent back with an error
    let tool_results = mock_transport.get_tool_results_sent();
    assert_eq!(tool_results.len(), 1);
    assert_eq!(tool_results[0].0, "inv-2");
    assert_eq!(
        tool_results[0].1,
        json!({"error": "Tool error: Tool failed intentionally"})
    );

    Ok(())
}

#[tokio::test]
async fn test_chat_stream_tool_timeout() -> Result<()> {
    struct MockSlowTool;

    #[async_trait::async_trait]
    impl Tool for MockSlowTool {
        fn name(&self) -> &str {
            "slow_tool"
        }

        async fn call(
            &self,
            _input: serde_json::Value,
            _ctx: ToolContext,
        ) -> Result<serde_json::Value> {
            tokio::time::sleep(Duration::from_secs(5)).await; // Longer than max_tool_runtime
            Ok(json!({"result": "Too slow"}))
        }
    }

    let mock_transport = Arc::new(MockTransport::new().with_chat_responses(vec![
        StreamEvent::ToolCall {
            invocation_id: "inv-3".to_string(),
            name: "slow_tool".to_string(),
            input: json!({}),
        },
        StreamEvent::Done {
            final_message: None,
        },
    ]));

    let mut client = OllamaClient::builder()
        .base_url("http://mock.ollama.ai")
        .max_tool_runtime(Duration::from_millis(20)) // Very short timeout
        .transport(mock_transport.clone())
        .build()?;

    client.register_tool(Arc::new(MockSlowTool))?;

    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "Run slow tool".to_string(),
            name: None,
            metadata: None,
        }],
        stream: Some(true),
        max_tokens: None,
        temperature: None,
        tools: None,
        request_id: None,
    };

    let mut stream = client.chat_stream(request).await?;
    let mut tool_call_received = false;

    while let Some(event_res) = stream.next().await {
        let event = event_res?;
        if let StreamEvent::ToolCall { .. } = event {
            tool_call_received = true;
        }
    }

    assert!(tool_call_received);

    // Give spawned task time to complete
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify that the tool result was sent back with a timeout error
    let tool_results = mock_transport.get_tool_results_sent();
    assert_eq!(tool_results.len(), 1);
    assert_eq!(tool_results[0].0, "inv-3");
    assert!(tool_results[0].1["error"]
        .as_str()
        .unwrap()
        .contains("timed out"));

    Ok(())
}
