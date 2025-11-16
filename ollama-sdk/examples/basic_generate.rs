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
