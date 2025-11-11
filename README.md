# Ollama-rs

An idiomatic Rust library for interacting with the Ollama API, focusing on streaming, tool calling, and ease of use.

## Features

*   **Idiomatic Rust API:** Designed with Rust's best practices in mind.
*   **Streaming Responses:** Efficiently handle streaming responses from the Ollama API.
*   **Tool Calling Support:** Seamless integration with Ollama's tool calling capabilities.
*   **Configurable Transport:** Uses `reqwest` by default, with an extensible `Transport` trait for custom implementations.
*   **Robust Error Handling:** Comprehensive error types for predictable error management.
*   **Observability:** Optional `tracing` for detailed logging and `metrics` for performance monitoring.

## Installation

> Note that: `ollama-rs` is not yet published on crates.io. You need to include it via Git.

Add `ollama-rs` to your `Cargo.toml` file:

```toml
[dependencies]
ollama-rs = { git = "https://github.com/Sathiyaraman-M/Ollama-rs.git", branch = "main" }
```

To enable optional features like `tracing` or `metrics`:

```toml
[dependencies]
ollama-rs = { git = "https://github.com/Sathiyaraman-M/Ollama-rs.git", branch = "main", features = ["tracing", "metrics"] }
```

## Usage

### Basic Chat (non-streaming)

```rust
use ollama_rs::OllamaClient;
use ollama_rs::types::Message;
use ollama_rs::types::Role;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::new_from_env();

    let messages = vec![
        Message::new(Role::User, "What is the capital of France?".to_string()),
    ];

    let chat_response = client.chat("llama2", messages).await?;

    println!("Response: {}", chat_response.content);

    Ok(())
}
```

### Streaming Chat

```rust
use ollama_rs::OllamaClient;
use ollama_rs::types::Message;
use ollama_rs::types::Role;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::new_from_env();

    let messages = vec![
        Message::new(Role::User, "Tell me a story about a Rust programmer.".to_string()),
    ];

    let mut stream = client.chat_stream("llama2", messages).await?;

    while let Some(event) = stream.next().await {
        match event {
            Ok(stream_event) => {
                if let Some(content) = stream_event.content {
                    print!("{}", content);
                }
            },
            Err(e) => eprintln!("Error: {}", e),
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
