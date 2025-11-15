use futures::StreamExt;
use ollama_sdk::types::chat::ChatRequestMessage;
use ollama_sdk::types::chat::ChatStreamEvent;
use ollama_sdk::types::chat::StreamingChatRequest;
use ollama_sdk::types::Role;
use ollama_sdk::OllamaClient;

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
