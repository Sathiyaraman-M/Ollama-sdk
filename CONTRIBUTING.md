# Contributing

## How to Contribute

### Bug Reports

If you find a bug, please open an issue on our [GitHub Issues page](https://github.com/Sathiyaraman-M/Ollama-rs/issues). When reporting a bug, please include:

*   A clear and concise description of the bug.
*   Steps to reproduce the behavior.
*   Expected behavior.
*   Actual behavior.
*   Any relevant error messages or logs.
*   Your operating system version, Ollama API version and Rust version.

### Feature Requests

If you have an idea for a new feature or enhancement, please open an issue on our [GitHub Issues page](https://github.com/Sathiyaraman-M/Ollama-rs/issues). Describe your idea clearly and explain why you think it would be a valuable addition to the library.

### Code Contributions

1.  **Fork the Repository:** Start by forking the `ollama-rs` repository to your GitHub account.
2.  **Clone Your Fork:** Clone your forked repository to your local machine:
    ```bash
    git clone https://github.com/Sathiyaraman-M/Ollama-rs.git
    cd ollama-rs
    ```
3.  **Create a New Branch:** Create a new branch for your feature or bug fix:
    ```bash
    git checkout -b feature/your-feature-name
    # or
    git checkout -b bugfix/your-bug-fix-name
    ```
4.  **Make Your Changes:** Implement your changes, ensuring they adhere to the existing code style and conventions.
5.  **Write Tests:** Add unit tests for your new features or bug fixes to ensure correctness and prevent regressions.
6.  **Run Tests:** Before submitting, make sure all tests pass:
    ```bash
    cargo test
    ```
7.  **Format and Lint:** Ensure your code is properly formatted and passes lint checks:
    ```bash
    cargo fmt --all
    cargo clippy --all-targets --all-features
    ```
8.  **Commit Your Changes:** Write clear and concise commit messages.
    ```bash
    git commit -m "feat: Add new feature"
    # or
    git commit -m "fix: Fix bug in streaming parser"
    ```
9.  **Push to Your Fork:** Push your changes to your forked repository:
    ```bash
    git push origin feature/your-feature-name
    ```
10. **Create a Pull Request:** Open a pull request from your fork to the `main` branch of the `ollama-rs` repository. Provide a detailed description of your changes.

## Development Environment

*   **Rust Toolchain:** Ensure you have a recent stable Rust toolchain installed. You can install it using `rustup`.
*   **Dependencies:** The project's dependencies are managed by `Cargo.toml`.

## License

By contributing to `ollama-rs`, you agree that your contributions will be licensed under the MIT License.

Thank you for contributing!
