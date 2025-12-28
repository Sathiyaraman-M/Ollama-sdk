use bytes::Bytes;
use futures::{stream, Stream, StreamExt};
use ollama_sdk::parser::{GenericStreamParser, StreamEventExt};
use ollama_sdk::types::chat::{ChatResponse, ChatStreamEvent};
use ollama_sdk::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MockMessage {
    id: String,
    content: String,
}

#[derive(Debug, PartialEq)]
enum MockStreamEvent {
    Message(MockMessage),
    Error(String),
    Partial {
        partial: String,
        error: Option<String>,
    },
}

impl StreamEventExt<MockMessage> for MockStreamEvent {
    fn from_message(msg: MockMessage) -> Self {
        MockStreamEvent::Message(msg)
    }

    fn from_error(err: String) -> Self {
        MockStreamEvent::Error(err)
    }

    fn partial(partial: String, error: Option<String>) -> Self {
        MockStreamEvent::Partial { partial, error }
    }
}

// --- Helper for creating a mock byte stream ---

fn mock_byte_stream(chunks: Vec<&str>) -> impl Stream<Item = Result<Bytes>> {
    stream::iter(
        chunks
            .into_iter()
            .map(|s| Ok(Bytes::from(s.to_string())))
            .collect::<Vec<Result<Bytes>>>(),
    )
}

#[tokio::test]
async fn test_generic_parser_single_full_message() {
    let raw_response = r#"{"id": "1", "content": "hello"}"#;
    let stream = mock_byte_stream(vec![&format!("{}\n", raw_response)]);
    let mut parser = GenericStreamParser::<_, MockMessage, MockStreamEvent>::new(stream);

    let event = parser.next().await.unwrap().unwrap();
    assert_eq!(
        event,
        MockStreamEvent::Message(MockMessage {
            id: "1".to_string(),
            content: "hello".to_string(),
        })
    );
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_generic_parser_multiple_full_messages() {
    let raw_response1 = r#"{"id": "1", "content": "first"}"#;
    let raw_response2 = r#"{"id": "2", "content": "second"}"#;
    let stream = mock_byte_stream(vec![
        &format!("{}\n", raw_response1),
        &format!("{}\n", raw_response2),
    ]);
    let mut parser = GenericStreamParser::<_, MockMessage, MockStreamEvent>::new(stream);

    let event1 = parser.next().await.unwrap().unwrap();
    assert_eq!(
        event1,
        MockStreamEvent::Message(MockMessage {
            id: "1".to_string(),
            content: "first".to_string(),
        })
    );

    let event2 = parser.next().await.unwrap().unwrap();
    assert_eq!(
        event2,
        MockStreamEvent::Message(MockMessage {
            id: "2".to_string(),
            content: "second".to_string(),
        })
    );
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_generic_parser_ollama_error() {
    let error_response = r#"{"error": "something went wrong"}"#;
    let stream = mock_byte_stream(vec![&format!("{}\n", error_response)]);
    let mut parser = GenericStreamParser::<_, MockMessage, MockStreamEvent>::new(stream);

    let event = parser.next().await.unwrap().unwrap();
    assert_eq!(
        event,
        MockStreamEvent::Error("something went wrong".to_string())
    );
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_generic_parser_partial_message() {
    // Malformed JSON that can't be deserialized as MockMessage or OllamaError
    let raw_partial = r#"{id: "1", content: "hello""#;
    let stream = mock_byte_stream(vec![&format!("{}\n", raw_partial)]);
    let mut parser = GenericStreamParser::<_, MockMessage, MockStreamEvent>::new(stream);

    let event = parser.next().await.unwrap().unwrap();
    assert!(matches!(event, MockStreamEvent::Partial { partial, .. } if partial == raw_partial));
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_generic_parser_mixed_messages() {
    let raw_response1 = r#"{"id": "1", "content": "first"}"#;
    let raw_partial = r#"{malformed json"#;
    let error_response = r#"{"error": "critical failure"}"#;
    let raw_response2 = r#"{"id": "2", "content": "second"}"#;

    let stream = mock_byte_stream(vec![
        &format!("{}\n", raw_response1),
        &format!("{}\n", raw_partial),
        &format!("{}\n", error_response),
        &format!("{}\n", raw_response2),
    ]);
    let mut parser = GenericStreamParser::<_, MockMessage, MockStreamEvent>::new(stream);

    let event = parser.next().await.unwrap().unwrap();
    assert_eq!(
        event,
        MockStreamEvent::Message(MockMessage {
            id: "1".to_string(),
            content: "first".to_string(),
        })
    );

    let event = parser.next().await.unwrap().unwrap();
    assert!(matches!(event, MockStreamEvent::Partial { partial, .. } if partial == raw_partial));

    let event = parser.next().await.unwrap().unwrap();
    assert_eq!(
        event,
        MockStreamEvent::Error("critical failure".to_string())
    );

    let event = parser.next().await.unwrap().unwrap();
    assert_eq!(
        event,
        MockStreamEvent::Message(MockMessage {
            id: "2".to_string(),
            content: "second".to_string(),
        })
    );
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_generic_parser_empty_lines_skipped() {
    let raw_response = r#"{"id": "1", "content": "hello"}"#;
    let stream = mock_byte_stream(vec!["\n", "\n", &format!("{}\n", raw_response), "\n"]);
    let mut parser = GenericStreamParser::<_, MockMessage, MockStreamEvent>::new(stream);

    let event = parser.next().await.unwrap().unwrap();
    assert_eq!(
        event,
        MockStreamEvent::Message(MockMessage {
            id: "1".to_string(),
            content: "hello".to_string(),
        })
    );
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_generic_parser_message_split_across_chunks() {
    let _raw_response = r#"{"id": "1", "content": "hello world"}"#;
    let stream = mock_byte_stream(vec![
        r#"{"id": "1", "content": "hello "#,
        r#"world"}"#,
        "\n",
    ]);
    let mut parser = GenericStreamParser::<_, MockMessage, MockStreamEvent>::new(stream);

    let event = parser.next().await.unwrap().unwrap();
    assert_eq!(
        event,
        MockStreamEvent::Message(MockMessage {
            id: "1".to_string(),
            content: "hello world".to_string(),
        })
    );
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_generic_parser_stream_ends_with_partial() {
    let raw_partial = r#"{"id": "1", "content": "incomplete"#;
    let stream = mock_byte_stream(vec![raw_partial]); // No trailing newline
    let mut parser = GenericStreamParser::<_, MockMessage, MockStreamEvent>::new(stream);

    let event = parser.next().await.unwrap().unwrap();
    assert_eq!(
        event,
        MockStreamEvent::Partial {
            partial: raw_partial.to_string(),
            error: None,
        }
    );
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_generic_parser_complex_partial_with_newline_then_complete() {
    let raw_partial = r#"{"id": "1", "content": "first part"#;
    let raw_response = r#"{"id": "2", "content": "second full"}"#;
    let stream = mock_byte_stream(vec![
        &format!("{}\n", raw_partial), // Partial followed by newline
        &format!("{}\n", raw_response),
    ]);
    let mut parser = GenericStreamParser::<_, MockMessage, MockStreamEvent>::new(stream);

    // The first one should still be a partial because it's malformed JSON
    let event1 = parser.next().await.unwrap().unwrap();
    assert!(matches!(event1, MockStreamEvent::Partial { partial, .. } if partial == raw_partial));

    let event2 = parser.next().await.unwrap().unwrap();
    assert_eq!(
        event2,
        MockStreamEvent::Message(MockMessage {
            id: "2".to_string(),
            content: "second full".to_string(),
        })
    );
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_chat_parser_tool_call_invocation() {
    // This test verifies that a chat-stream line containing a tool invocation
    // in the shape returned by the model is parsed correctly into the
    // ChatResponse/ToolCall structures.
    let raw = r#"{"model":"llama3.2:3b","created_at":"2025-12-25T14:18:57.522402Z","message":{"role":"assistant","content":"","tool_calls":[{"id":"call_etugzc4r","function":{"index":0,"name":"","arguments":{"n":"10"}}}]},"done":false}"#;
    let stream = mock_byte_stream(vec![&format!("{}\n", raw)]);
    let mut parser = GenericStreamParser::<_, ChatResponse, ChatStreamEvent>::new(stream);

    let event = parser.next().await.unwrap().unwrap();
    match event {
        ChatStreamEvent::Message(resp) => {
            assert_eq!(resp.model, "llama3.2:3b".to_string());
            assert!(!resp.message.tool_calls.is_empty());

            let tool_call = &resp.message.tool_calls[0];

            assert_eq!(resp.message.tool_calls.len(), 1);
            assert_eq!(tool_call.id, "call_etugzc4r");
            assert_eq!(tool_call.function.index, Some(0));
            // Name is present but empty in this invocation
            assert!(tool_call.function.name.is_empty());
            assert_eq!(
                tool_call
                    .function
                    .arguments
                    .get("n")
                    .unwrap()
                    .as_str()
                    .unwrap(),
                "10"
            );
        }
        _ => panic!("expected message event"),
    }

    assert!(parser.next().await.is_none());
}
