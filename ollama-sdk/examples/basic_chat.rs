use ollama_sdk::{
    types::{
        chat::{ChatRequestMessage, SimpleChatRequest},
        Role,
    },
    OllamaClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::builder().build()?;

    let model = "llama3.2:3b".to_string();
    let messages = vec![ChatRequestMessage::new(
        Role::User,
        "What is the capital of France".to_string(),
    )];

    let chat_request = SimpleChatRequest::new(model, messages);

    let chat_response = client.chat_simple(chat_request).await?;
    let message = chat_response.message;

    println!("Response: {}", message.content);

    Ok(())
}
