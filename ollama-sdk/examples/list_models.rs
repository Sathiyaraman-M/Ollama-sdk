use ollama_sdk::OllamaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let response = client.list_models().await?;

    for model in response.models {
        println!("{}", model.name);
    }

    Ok(())
}
