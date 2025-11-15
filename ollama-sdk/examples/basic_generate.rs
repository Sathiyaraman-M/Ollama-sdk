use ollama_sdk::types::generate::SimpleGenerateRequest;
use ollama_sdk::OllamaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let generate_request = SimpleGenerateRequest {
        model: "llama3.2:3b".to_string(),
        prompt: "Tell me a story about a Rust programmer."
            .to_string()
            .into(),
        ..Default::default()
    };

    let generate_response = client.generate_simple(generate_request).await?;

    println!("Response: {}", generate_response.response);

    Ok(())
}
