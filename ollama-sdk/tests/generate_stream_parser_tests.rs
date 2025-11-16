use bytes::Bytes;
use futures::stream;
use futures::StreamExt;
use ollama_sdk::stream::GenerateStreamParser;
use ollama_sdk::types::generate::{GenerateResponse, GenerateStreamEvent};
use ollama_sdk::Result;

// Helper function to create a stream from a vector of byte chunks
fn create_byte_stream(
    chunks: Vec<String>,
) -> impl futures::Stream<Item = Result<Bytes>> + Send + 'static {
    stream::iter(chunks.into_iter().map(|s| Ok(Bytes::from(s))))
}

#[tokio::test]
async fn test_parse_single_message_chunk_event() {
    let json_line = r#"{"model":"llama2","created_at":"2023-08-04T19:22:45.499127Z","response":"Hello","done":false}"#.to_string();
    let byte_stream = create_byte_stream(vec![format!("{}\n", json_line)]);
    let mut parser = GenerateStreamParser::new(byte_stream);
    let event = parser.next().await.unwrap().unwrap();
    match event {
        GenerateStreamEvent::MessageChunk(GenerateResponse { response, .. }) => {
            assert_eq!(response, "Hello");
        }
        _ => panic!("Expected MessageChunk event, got {:?}", event),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_parse_multiple_message_chunk_events() {
    let json_line1 = r#"{"model":"llama2","created_at":"2023-08-04T19:22:45.499127Z","response":"Hello","done":false}"#.to_string();
    let json_line2 = r#"{"model":"llama2","created_at":"2023-08-04T19:22:45.599127Z","response":" World","done":false}"#.to_string();
    let byte_stream = create_byte_stream(vec![
        format!("{}\n", json_line1),
        format!("{}\n", json_line2),
    ]);
    let mut parser = GenerateStreamParser::new(byte_stream);
    let event1 = parser.next().await.unwrap().unwrap();
    match event1 {
        GenerateStreamEvent::MessageChunk(GenerateResponse { response, .. }) => {
            assert_eq!(response, "Hello");
        }
        _ => panic!("Expected MessageChunk event"),
    }
    let event2 = parser.next().await.unwrap().unwrap();
    match event2 {
        GenerateStreamEvent::MessageChunk(GenerateResponse { response, .. }) => {
            assert_eq!(response, " World");
        }
        _ => panic!("Expected MessageChunk event"),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_parse_error_event() {
    let json_line = r#"{"error":"something went wrong"}"#.to_string();
    let byte_stream = create_byte_stream(vec![format!("{}\n", json_line)]);
    let mut parser = GenerateStreamParser::new(byte_stream);
    let event = parser.next().await.unwrap().unwrap();
    match event {
        GenerateStreamEvent::Error(error) => {
            assert_eq!(error, "something went wrong");
        }
        _ => panic!("Expected Error event, got {:?}", event),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_handle_incomplete_lines_and_buffering() {
    let json_line1 = r#"{"model":"llama2","created_at":"2023-08-04T19:22:45.499127Z","response":"Hello","done":false}"#.to_string();
    let json_line2 = r#"{"model":"llama2","created_at":"2023-08-04T19:22:45.599127Z","response":" World","done":false}"#.to_string();
    let byte_stream = create_byte_stream(vec![
        json_line1[..10].to_string(), // Incomplete first line
        format!("{}\n{}", &json_line1[10..], json_line2),
        // Rest of first line + incomplete second line
        "\n".to_string(), // Newline for second line
    ]);
    let mut parser = GenerateStreamParser::new(byte_stream);
    let event1 = parser.next().await.unwrap().unwrap();
    match event1 {
        GenerateStreamEvent::MessageChunk(GenerateResponse { response, .. }) => {
            assert_eq!(response, "Hello");
        }
        _ => panic!("Expected MessageChunk event"),
    }
    let event2 = parser.next().await.unwrap().unwrap();
    match event2 {
        GenerateStreamEvent::MessageChunk(GenerateResponse { response, .. }) => {
            assert_eq!(response, " World");
        }
        _ => panic!("Expected MessageChunk event"),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_handle_non_json_lines_as_partial() {
    let non_json_line = "This is a plain text message.".to_string();
    let json_line = r#"{"model":"llama2","created_at":"2023-08-04T19:22:45.499127Z","response":"JSON part","done":false}"#.to_string();
    let byte_stream = create_byte_stream(vec![
        format!("{}\n", non_json_line),
        format!("{}\n", json_line),
    ]);
    let mut parser = GenerateStreamParser::new(byte_stream);
    let event1 = parser.next().await.unwrap().unwrap();
    match event1 {
        GenerateStreamEvent::Partial { partial, .. } => {
            assert_eq!(partial, "This is a plain text message.");
        }
        _ => panic!("Expected Partial event for non-JSON line"),
    }
    let event2 = parser.next().await.unwrap().unwrap();
    match event2 {
        GenerateStreamEvent::MessageChunk(GenerateResponse { response, .. }) => {
            assert_eq!(response, "JSON part");
        }
        _ => panic!("Expected MessageChunk event for JSON line"),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_empty_stream() {
    let byte_stream = create_byte_stream(vec![]);
    let mut parser = GenerateStreamParser::new(byte_stream);
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_stream_with_empty_lines() {
    let json_line = r#"{"model":"llama2","created_at":"2023-08-04T19:22:45.499127Z","response":"test","done":false}"#.to_string();
    let byte_stream = create_byte_stream(vec![
        "\n".to_string(),
        format!("{}\n", json_line),
        "\n\n".to_string(),
    ]);
    let mut parser = GenerateStreamParser::new(byte_stream);
    let event = parser.next().await.unwrap().unwrap();
    match event {
        GenerateStreamEvent::MessageChunk(GenerateResponse { response, .. }) => {
            assert_eq!(response, "test");
        }
        _ => panic!("Expected MessageChunk event"),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_stream_ends_with_partial_line() {
    let partial_line = "This is a partial line".to_string();
    let byte_stream = create_byte_stream(vec![
        partial_line.clone(), // No trailing newline
    ]);
    let mut parser = GenerateStreamParser::new(byte_stream);
    let event = parser.next().await.unwrap().unwrap();
    match event {
        GenerateStreamEvent::Partial { partial, .. } => {
            assert_eq!(partial, partial_line);
        }
        _ => panic!("Expected Partial event"),
    }
    assert!(parser.next().await.is_none());
}
