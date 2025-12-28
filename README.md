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
ollama-sdk = "0.4.0"
```

To enable optional features like `tracing` or `metrics`:

```toml
[dependencies]
ollama-sdk = { version = "0.4.0", features = ["tracing", "metrics"] }
```

## Examples

> [!TIP]
> Examples are present in the [examples](./ollama-sdk/examples) directory. You can run them using `cargo run --example <example_name>`.

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
