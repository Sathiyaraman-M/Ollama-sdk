use ollama_sdk::types::chat::ChatRequestMessage;
use ollama_sdk::types::chat::SimpleChatRequest;
use ollama_sdk::types::Role;
use ollama_sdk::OllamaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let messages = vec![ChatRequestMessage {
        role: Role::User,
        content: "What is the capital of France".to_string(),
        ..Default::default()
    }];

    let chat_request = SimpleChatRequest {
        model: "llama3.2:3b".to_string(),
        messages,
        ..Default::default()
    };

    let chat_response = client.chat_simple(chat_request).await?;
    let message = chat_response.message;

    println!("Response: {}", message.content);

    Ok(())
}
