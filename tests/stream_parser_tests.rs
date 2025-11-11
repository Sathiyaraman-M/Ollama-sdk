use bytes::Bytes;
use futures::{stream, StreamExt};
use serde_json::json;

use ollama_rs::errors::Result;
use ollama_rs::stream::parser::StreamParser;
use ollama_rs::types::{Role, StreamEvent};

// Helper function to create a stream from a vector of byte chunks
fn create_byte_stream(
    chunks: Vec<String>,
) -> impl futures::Stream<Item = Result<Bytes>> + Send + 'static {
    stream::iter(chunks.into_iter().map(|s| Ok(Bytes::from(s))))
}

#[tokio::test]
async fn test_parse_single_partial_event() {
    let json_line = r#"{"Partial":{"message":{"role":"assistant","content":"hello"}}}"#.to_string();
    let byte_stream = create_byte_stream(vec![format!("{}\n", json_line)]);
    let mut parser = StreamParser::new(byte_stream);

    let event = parser.next().await.unwrap().unwrap();
    match event {
        StreamEvent::Partial { message } => {
            assert_eq!(message.role, Role::Assistant);
            assert_eq!(message.content, "hello");
        }
        _ => panic!("Expected Partial event, got {:?}", event),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_parse_multiple_partial_events() {
    let json_line1 =
        r#"{"Partial":{"message":{"role":"assistant","content":"hello"}}}"#.to_string();
    let json_line2 =
        r#"{"Partial":{"message":{"role":"assistant","content":" world"}}}"#.to_string();
    let byte_stream = create_byte_stream(vec![
        format!("{}\n", json_line1),
        format!("{}\n", json_line2),
    ]);
    let mut parser = StreamParser::new(byte_stream);

    let event1 = parser.next().await.unwrap().unwrap();
    match event1 {
        StreamEvent::Partial { message } => assert_eq!(message.content, "hello"),
        _ => panic!("Expected Partial event"),
    }

    let event2 = parser.next().await.unwrap().unwrap();
    match event2 {
        StreamEvent::Partial { message } => assert_eq!(message.content, " world"),
        _ => panic!("Expected Partial event"),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_parse_tool_call_event() {
    let json_line =
        r#"{"ToolCall":{"invocation_id":"123","name":"search","input":{"query":"rust"}}}"#
            .to_string();
    let byte_stream = create_byte_stream(vec![format!("{}\n", json_line)]);
    let mut parser = StreamParser::new(byte_stream);

    let event = parser.next().await.unwrap().unwrap();
    match event {
        StreamEvent::ToolCall {
            invocation_id,
            name,
            input,
        } => {
            assert_eq!(invocation_id, "123");
            assert_eq!(name, "search");
            assert_eq!(input, json!({ "query": "rust" }));
        }
        _ => panic!("Expected ToolCall event, got {:?}", event),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_parse_done_event() {
    let json_line =
        r#"{"Done":{"final_message":{"role":"assistant","content":"finished"}}}"#.to_string();
    let byte_stream = create_byte_stream(vec![format!("{}\n", json_line)]);
    let mut parser = StreamParser::new(byte_stream);

    let event = parser.next().await.unwrap().unwrap();
    match event {
        StreamEvent::Done { final_message } => {
            assert_eq!(final_message.unwrap().content, "finished");
        }
        _ => panic!("Expected Done event, got {:?}", event),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_handle_incomplete_lines_and_buffering() {
    let json_line1 =
        r#"{"Partial":{"message":{"role":"assistant","content":"hello"}}}"#.to_string();
    let json_line2 =
        r#"{"Partial":{"message":{"role":"assistant","content":" world"}}}"#.to_string();

    let byte_stream = create_byte_stream(vec![
        json_line1[..10].to_string(), // Incomplete first line
        format!("{}\n{}", &json_line1[10..], json_line2), // Rest of first line + incomplete second line
        "\n".to_string(),                                 // Newline for second line
    ]);
    let mut parser = StreamParser::new(byte_stream);

    let event1 = parser.next().await.unwrap().unwrap();
    match event1 {
        StreamEvent::Partial { message } => assert_eq!(message.content, "hello"),
        _ => panic!("Expected Partial event"),
    }

    let event2 = parser.next().await.unwrap().unwrap();
    match event2 {
        StreamEvent::Partial { message } => assert_eq!(message.content, " world"),
        _ => panic!("Expected Partial event"),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_handle_non_json_lines_as_partial() {
    let non_json_line = "This is a plain text message.".to_string();
    let json_line =
        r#"{"Partial":{"message":{"role":"assistant","content":"JSON part"}}}"#.to_string();

    let byte_stream = create_byte_stream(vec![
        format!("{}\n", non_json_line),
        format!("{}\n", json_line),
    ]);
    let mut parser = StreamParser::new(byte_stream);

    let event1 = parser.next().await.unwrap().unwrap();
    match event1 {
        StreamEvent::Partial { message } => {
            assert_eq!(message.role, Role::Assistant);
            assert_eq!(message.content, "This is a plain text message.");
        }
        _ => panic!("Expected Partial event for non-JSON line"),
    }

    let event2 = parser.next().await.unwrap().unwrap();
    match event2 {
        StreamEvent::Partial { message } => {
            assert_eq!(message.role, Role::Assistant);
            assert_eq!(message.content, "JSON part");
        }
        _ => panic!("Expected Partial event for JSON line"),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_empty_stream() {
    let byte_stream = create_byte_stream(vec![]);
    let mut parser = StreamParser::new(byte_stream);
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_stream_with_empty_lines() {
    let json_line = r#"{"Partial":{"message":{"role":"assistant","content":"test"}}}"#.to_string();
    let byte_stream = create_byte_stream(vec![
        "\n".to_string(),
        format!("{}\n", json_line),
        "\n\n".to_string(),
    ]);
    let mut parser = StreamParser::new(byte_stream);

    let event = parser.next().await.unwrap().unwrap();
    match event {
        StreamEvent::Partial { message } => assert_eq!(message.content, "test"),
        _ => panic!("Expected Partial event"),
    }
    assert!(parser.next().await.is_none());
}

#[tokio::test]
async fn test_stream_ends_with_partial_line() {
    let json_line = r#"{"Partial":{"message":{"role":"assistant","content":"hello"}}}"#.to_string();
    let byte_stream = create_byte_stream(vec![
        json_line[..10].to_string(), // Incomplete first line
        json_line[10..].to_string(), // Rest of first line, but no trailing newline
    ]);
    let mut parser = StreamParser::new(byte_stream);

    let event = parser.next().await.unwrap().unwrap();
    match event {
        StreamEvent::Partial { message } => {
            assert_eq!(message.content, json_line);
        }
        _ => panic!("Expected Partial event"),
    }
    assert!(parser.next().await.is_none());
}
