use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use ollama_sdk::{
    tools::{Tool, ToolContext},
    types::{
        chat::{
            ChatRequestMessage, ChatStreamEvent, FunctionalTool, StreamingChatRequest, ToolCall,
            ToolSpec,
        },
        Role,
    },
    Error, OllamaClient,
};

struct FibonacciTool;

impl FibonacciTool {
    fn parse_n(input: &Value) -> Result<u64, Error> {
        input
            .get("n")
            .and_then(|v| {
                if v.is_u64() {
                    v.as_u64()
                } else if v.is_i64() {
                    v.as_i64()
                        .and_then(|i| if i >= 0 { Some(i as u64) } else { None })
                } else if v.is_string() {
                    v.as_str().and_then(|s| s.parse::<u64>().ok())
                } else {
                    None
                }
            })
            .ok_or_else(|| Error::Tool("Missing or invalid parameter 'n'".to_string()))
    }

    fn fibonacci(n: u64) -> u128 {
        match n {
            0 => 0,
            1 => 1,
            _ => {
                let mut a: u128 = 0;
                let mut b: u128 = 1;
                for _ in 2..=n {
                    let c = a + b;
                    a = b;
                    b = c;
                }
                b
            }
        }
    }
}

#[async_trait]
impl Tool for FibonacciTool {
    fn name(&self) -> &str {
        "fibonacci"
    }

    async fn call(&self, input: Value, _ctx: ToolContext) -> std::result::Result<Value, Error> {
        let n = Self::parse_n(&input)?;

        Ok(serde_json::json!({ "result": Self::fibonacci(n) }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OllamaClient::builder().build()?;
    let fib_tool = Arc::new(FibonacciTool);
    client.register_tool(fib_tool.clone())?;

    let model = "llama3.2:3b".to_string();

    let fib_tool_spec = FunctionalTool {
        name: "fibonacci".to_string(),
        description: Some(
            "Compute the Fibonacci number for a given non-negative integer `n`".to_string(),
        ),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "n": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Index in the Fibonacci sequence (0-based)."
                }
            },
            "required": ["n"]
        }),
    };

    let user_prompt = format!(
        "Use the `fibonacci` function to compute fibonacci(n) and return the value. \
        For example, compute fibonacci for n=31 and n=11. Make sure to call the function \
        instead of trying to compute it yourself or making up values."
    );

    let mut history = vec![ChatRequestMessage::new(Role::User, user_prompt.clone())];

    let tools = vec![ToolSpec::Function(fib_tool_spec.clone())];

    println!("user: {}", user_prompt);

    let mut tool_map = std::collections::HashMap::new();
    tool_map.insert(fib_tool.name().to_string(), fib_tool.clone());

    loop {
        let request =
            StreamingChatRequest::new(model.clone(), history.clone()).tools(tools.clone());
        let mut stream = client.chat_stream(request).await?;

        let mut tool_was_called = false;
        let mut assistant_prefix_printed = false;

        while let Some(event_res) = stream.next().await {
            match event_res {
                Ok(event) => match event {
                    ChatStreamEvent::Message(response) => {
                        if !assistant_prefix_printed {
                            print!("assistant: ");
                            assistant_prefix_printed = true;
                        }
                        print!("{}", response.message.content);

                        history.push(response.message.clone().into());

                        if !response.message.tool_calls.is_empty() {
                            for call in response.message.tool_calls.iter() {
                                let (maybe_name, params_value) = match call {
                                    ToolCall::Invocation { function, .. } => (
                                        function.name.as_ref().and_then(|s| {
                                            if s.is_empty() {
                                                None
                                            } else {
                                                Some(s.clone())
                                            }
                                        }),
                                        function.arguments.clone(),
                                    ),
                                    ToolCall::Function(f) => {
                                        (Some(f.name.clone()), f.parameters.clone())
                                    }
                                };

                                let tool_name = if let Some(n) = maybe_name {
                                    n
                                } else if let ToolCall::Invocation { function, .. } = call {
                                    if let Some(idx) = function.index {
                                        if let Some(ToolSpec::Function(ft)) = tools.get(idx) {
                                            ft.name.clone()
                                        } else {
                                            "<unknown>".to_string()
                                        }
                                    } else {
                                        "<unknown>".to_string()
                                    }
                                } else {
                                    "<unknown>".to_string()
                                };

                                println!("tool_call: {} params: {}", tool_name, params_value);

                                let params = params_value.clone();
                                let ctx = ToolContext {
                                    cancellation_token: CancellationToken::new(),
                                };

                                if let Some(tool) = tool_map.get(&tool_name) {
                                    match tool.call(params.clone(), ctx).await {
                                        Ok(tool_result) => {
                                            println!(
                                                "tool_result: {} result: {}",
                                                tool_name, tool_result
                                            );

                                            let tool_msg = ChatRequestMessage::new(
                                                Role::Tool,
                                                serde_json::to_string(&tool_result)
                                                    .unwrap_or_else(|_| tool_result.to_string()),
                                            );
                                            history.push(tool_msg);

                                            tool_was_called = true;
                                            break;
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "tool_result: {} error: Tool invocation failed: {}",
                                                tool_name, e
                                            );

                                            let tool_msg = ChatRequestMessage::new(
                                                Role::Tool,
                                                format!("Tool error: {}", e),
                                            );
                                            history.push(tool_msg);

                                            tool_was_called = true;
                                            break;
                                        }
                                    }
                                } else {
                                    eprintln!(
                                        "tool_result: {} error: No registered tool named '{}'",
                                        tool_name, tool_name
                                    );
                                    let tool_msg = ChatRequestMessage::new(
                                        Role::Tool,
                                        format!("Tool '{}' not found", tool_name),
                                    );
                                    history.push(tool_msg);

                                    tool_was_called = true;
                                    break;
                                }
                            }

                            if tool_was_called {
                                break;
                            }
                        }

                        if response.done && response.message.tool_calls.is_empty() {
                            println!("assistant: [done]");
                            return Ok(());
                        }
                    }
                    ChatStreamEvent::Error(err) => {
                        println!("Error chunk from server: {}", err);
                    }
                    _ => {
                        println!("Unhandled event: {:?}", event);
                    }
                },
                Err(e) => {
                    eprintln!("Streaming error: {}", e);
                }
            }
        }

        if !tool_was_called {
            break;
        }
    }

    Ok(())
}
