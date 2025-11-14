# Ollama-sdk

An idiomatic Rust library for interacting with the Ollama API, focusing on streaming, tool calling, and ease of use.

> [!NOTE]
> This is not an official Ollama SDK.

> [!WARNING]
> This library is currently in pre-alpha so don't use it in production. Only generation and chat completions are implemented.

## Features

*   **Idiomatic Rust API:** Designed with Rust's best practices in mind.
*   **Streaming Responses:** Efficiently handle streaming responses from the Ollama API.
*   **Configurable Transport:** Uses `reqwest` by default, with an extensible `Transport` trait for custom implementations.
*   **Robust Error Handling:** Comprehensive error types for predictable error management.
*   **Observability:** Optional `tracing` for detailed logging and `metrics` for performance monitoring.

## Installation

Add `ollama-sdk` to your `Cargo.toml` file:

```toml
[dependencies]
ollama-sdk = "0.2.1"
```

To enable optional features like `tracing` or `metrics`:

```toml
[dependencies]
ollama-sdk = { version = "0.2.1", features = ["tracing", "metrics"] }
```

## Usage

### Basic Generation (non-streaming)

```rust
use ollama_sdk::OllamaClient;
use ollama_sdk::types::generate::SimpleGenerateRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let generate_request = SimpleGenerateRequest {
        model: "llama3.2:3b".to_string(),
        prompt: "Tell me a story about a Rust programmer.".to_string().into(),
        ..Default::default()
    };

    let generate_response = client.generate_simple(generate_request).await?;

    println!("Response: {}", generate_response.response);

    Ok(())
}
```

### Streaming Generation

```rust
use futures::StreamExt;
use ollama_sdk::OllamaClient;
use ollama_sdk::types::generate::GenerateStreamEvent;
use ollama_sdk::types::generate::StreamingGenerateRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let generate_request = StreamingGenerateRequest {
        model: "llama3.2:3b".to_string(),
        prompt: "Tell me a story about a Rust programmer.".to_string().into(),
        ..Default::default()
    };

    let mut stream = client.generate_stream(generate_request).await?;

    while let Some(event) = stream.next().await {
        match event {
            Ok(val) => match val {
                GenerateStreamEvent::MessageChunk(chunk) => print!("{}", chunk.response),
                GenerateStreamEvent::Error(error) => println!("\nError Chunk: {}", error),
                _ => continue,
            },
            Err(e) => eprintln!("Chat Error: {}", e),
        }
    }
    println!();

    Ok(())
}
```

### Basic Chat (non-streaming)

```rust
use ollama_sdk::OllamaClient;
use ollama_sdk::types::chat::ChatRequestMessage;
use ollama_sdk::types::chat::SimpleChatRequest;
use ollama_sdk::types::Role;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let messages = vec![ChatRequestMessage {
        role: Role::User,
        content: "What is the capital of France".to_string(),
    }];

    let chat_request = SimpleChatRequest {
        model: "llama3.2:3b".to_string(),
        messages,
        ..Default::default()
    };

    let chat_response = client.chat_simple(chat_request).await?;
    let message = chat_response.message;

    println!("Response: {}", message.content);

    Ok(())
}
```

### Streaming Chat

```rust
use ollama_sdk::OllamaClient;
use ollama_sdk::types::chat::ChatRequestMessage;
use ollama_sdk::types::chat::ChatStreamEvent;
use ollama_sdk::types::chat::StreamingChatRequest;
use ollama_sdk::types::Role;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let messages = vec![ChatRequestMessage {
        role: Role::User,
        content: "Tell me a story about a Rust programmer.".to_string(),
        ..Default::default()
    }];
    
    let chat_request = StreamingChatRequest {
        model: "llama3.2:3b".to_string(),
        messages,
        ..Default::default()
    };

    let mut stream = client.chat_stream(chat_request).await?;

    while let Some(event) = stream.next().await {
        match event {
            Ok(val) => match val {
                ChatStreamEvent::Message(response) => print!("{}", response.message.content),
                ChatStreamEvent::Error(error) => println!("\nError Chunk: {}", error),
                _ => continue,
            },
            Err(e) => eprintln!("Chat Error: {}", e),
        }
    }
    println!();

    Ok(())
}
```

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
