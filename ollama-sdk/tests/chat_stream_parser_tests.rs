use bytes::Bytes;
use futures::{stream, StreamExt};

use ollama_sdk::stream::chat_stream_parser::ChatStreamParser;
use ollama_sdk::types::chat::ChatStreamEvent;
use ollama_sdk::Result;

// Helper function to create a stream from a vector of byte chunks
fn create_byte_stream(
    chunks: Vec<String>,
) -> impl futures::Stream<Item = Result<Bytes>> + Send + 'static {
    stream::iter(chunks.into_iter().map(|s| Ok(Bytes::from(s))))
}

#[tokio::test]
async fn test_parse_single_message_event() {
    let json_line =
        r#"{"model":"gpt-2","message":{"role":"assistant","content":"hello"},"done":false}"#
            .to_string();
    let byte_stream = create_byte_stream(vec![format!("{}\n", json_line)]);
    let mut parser = ChatStreamParser::new(byte_stream);

    let event = parser.next().await.unwrap().unwrap();
    match event {
        ChatStreamEvent::Message(response) => assert_eq!(response.message.content, "hello"),
        _ => panic!("Expected Message event, got {:?}", event),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_parse_single_partial_event() {
    let json_line =
        r#"{"message":{"role":"assistant","content":"hello"},"done":false}"#.to_string();
    let byte_stream = create_byte_stream(vec![format!("{}\n", json_line)]);
    let mut parser = ChatStreamParser::new(byte_stream);

    let event = parser.next().await.unwrap().unwrap();
    match event {
        ChatStreamEvent::Partial { partial, .. } => assert_eq!(partial, json_line),
        _ => panic!("Expected Partial event, got {:?}", event),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_parse_single_error_event() {
    let json_line = r#"{"error":"Some test error"}"#.to_string();
    let byte_stream = create_byte_stream(vec![format!("{}\n", json_line)]);
    let mut parser = ChatStreamParser::new(byte_stream);

    let event = parser.next().await.unwrap().unwrap();
    match event {
        ChatStreamEvent::Error(err) => assert_eq!(err, "Some test error"),
        _ => panic!("Expected Error event, got {:?}", event),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_parse_multiple_message_events() {
    let json_line1 =
        r#"{"model":"gpt-2","message":{"role":"assistant","content":"hello"},"done":false}"#
            .to_string();
    let json_line2 =
        r#"{"model":"gpt-2","message":{"role":"assistant","content":" world"},"done":false}"#
            .to_string();
    let byte_stream = create_byte_stream(vec![
        format!("{}\n", json_line1),
        format!("{}\n", json_line2),
    ]);
    let mut parser = ChatStreamParser::new(byte_stream);

    let ev1 = parser.next().await.unwrap().unwrap();
    let ev2 = parser.next().await.unwrap().unwrap();

    match (ev1, ev2) {
        (ChatStreamEvent::Message(response1), ChatStreamEvent::Message(response2)) => {
            assert_eq!(response1.message.content, "hello");
            assert_eq!(response2.message.content, " world");
        }
        _ => panic!("Expected two Message events"),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_handle_mix_of_partial_and_message_events() {
    let non_json_line = "This is a plain text message.".to_string();
    let json_line =
        r#"{"model":"gpt-2","message":{"role":"assistant","content":"JSON part"},"done":false}"#
            .to_string();

    let byte_stream = create_byte_stream(vec![
        format!("{}\n", non_json_line),
        format!("{}\n", json_line),
    ]);
    let mut parser = ChatStreamParser::new(byte_stream);

    let ev1 = parser.next().await.unwrap().unwrap();
    let ev2 = parser.next().await.unwrap().unwrap();

    match (ev1, ev2) {
        (ChatStreamEvent::Partial { partial, .. }, ChatStreamEvent::Message(response2)) => {
            assert_eq!(partial, non_json_line);
            assert_eq!(response2.message.content, "JSON part");
        }
        _ => panic!("Expected one Partial and one Message events"),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_empty_stream() {
    let byte_stream = create_byte_stream(vec![]);
    let mut parser = ChatStreamParser::new(byte_stream);
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_stream_with_empty_lines() {
    let json_line =
        r#"{"model":"gpt-2","message":{"role":"assistant","content":"test"},"done":false}"#
            .to_string();
    let byte_stream = create_byte_stream(vec![
        "\n".to_string(),
        format!("{}\n", json_line),
        "\n\n".to_string(),
    ]);
    let mut parser = ChatStreamParser::new(byte_stream);

    let ev = parser.next().await.unwrap().unwrap();
    match ev {
        ChatStreamEvent::Message(response) => assert_eq!(response.message.content, "test"),
        _ => panic!("Expected Message event"),
    }
    assert!(parser.next().await.is_none());
}
