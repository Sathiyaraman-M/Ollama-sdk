use futures::StreamExt;
use ollama_sdk::{
    types::{
        chat::{ChatStreamEvent, RegularChatRequestMessage, StreamingChatRequest},
        Role,
    },
    OllamaClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let model = "llama3.2:3b".to_string();

    let message =
        RegularChatRequestMessage::new(Role::User, "What is the capital of France?".to_string());

    let chat_request = StreamingChatRequest::new(model).add_regular_message(message);

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
