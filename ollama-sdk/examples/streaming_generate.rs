use futures::StreamExt;
use ollama_sdk::types::generate::GenerateStreamEvent;
use ollama_sdk::types::generate::StreamingGenerateRequest;
use ollama_sdk::OllamaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let generate_request = StreamingGenerateRequest {
        model: "llama3.2:3b".to_string(),
        prompt: "Tell me a story about a Rust programmer."
            .to_string()
            .into(),
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
