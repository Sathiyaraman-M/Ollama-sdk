# Ollama-sdk

An idiomatic low-level Rust library for interacting with the Ollama API.

> [!NOTE]
> This is not an official Ollama SDK.

> [!WARNING]
> This library is currently in pre-alpha so don't use it in production. There are frequent breaking changes and the API is not stable.

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
ollama-sdk = "0.3.1"
```

To enable optional features like `tracing` or `metrics`:

```toml
[dependencies]
ollama-sdk = { version = "0.3.1", features = ["tracing", "metrics"] }
```

## Usage

> [!TIP]
> Examples are present in the [examples](./ollama-sdk/examples) directory

### Basic Generation (non-streaming)

```rust
use ollama_sdk::{types::generate::SimpleGenerateRequest, OllamaClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let model = "llama3.2:3b".to_string();
    let prompt = "Tell me a story about a Rust programmer.".to_string();

    let generate_request = SimpleGenerateRequest::new(model, prompt);

    let generate_response = client.generate_simple(generate_request).await?;

    println!("Response: {}", generate_response.response);

    Ok(())
}
```

### Streaming Generation

```rust
use futures::StreamExt;
use ollama_sdk::{
    types::generate::{GenerateStreamEvent, StreamingGenerateRequest},
    OllamaClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let model = "llama3.2:3b".to_string();
    let prompt = "Tell me a story about a Rust programmer.".to_string();

    let generate_request = StreamingGenerateRequest::new(model, prompt);

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

    Ok(())
}
```

### Basic Chat (non-streaming)

```rust
use ollama_sdk::{
    types::{
        chat::{ChatRequestMessage, SimpleChatRequest},
        Role,
    },
    OllamaClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let model = "llama3.2:3b".to_string();
    let messages = vec![ChatRequestMessage::new(
        Role::User,
        "What is the capital of France".to_string(),
    )];

    let chat_request = SimpleChatRequest::new(model, messages);

    let chat_response = client.chat_simple(chat_request).await?;
    let message = chat_response.message;

    println!("Response: {}", message.content);

    Ok(())
}
```

### Streaming Chat

```rust
use futures::StreamExt;
use ollama_sdk::{
    types::{
        chat::{ChatRequestMessage, ChatStreamEvent, StreamingChatRequest},
        Role,
    },
    OllamaClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let model = "llama3.2:3b".to_string();
    let messages = vec![ChatRequestMessage::new(
        Role::User,
        "Tell me a story about a Rust programmer.".to_string(),
    )];

    let chat_request = StreamingChatRequest::new(model, messages);

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

    Ok(())
}
```

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
