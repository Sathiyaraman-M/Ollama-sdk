use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use ollama_sdk::{
    tools::{Tool, ToolContext},
    types::{
        chat::{
            ChatRequestMessage, ChatStreamEvent, FunctionalTool, RegularChatRequestMessage,
            StreamingChatRequest, ToolCallResultMessage, ToolSpec,
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

    let mut history = vec![
        RegularChatRequestMessage::new(Role::User, user_prompt.clone()).to_chat_request_message(),
    ];

    let tools = vec![ToolSpec::Function {
        function: fib_tool_spec.clone(),
    }];

    println!("user: {}", user_prompt);

    let mut tool_map = std::collections::HashMap::new();
    tool_map.insert(fib_tool.name().to_string(), fib_tool.clone());

    loop {
        let mut request = StreamingChatRequest::new(model.clone()).tools(tools.clone());

        for msg in history.iter() {
            request = request.add_message(msg.clone());
        }

        let mut stream = client.chat_stream(request).await?;

        let mut tool_was_called = false;
        let mut assistant_prefix_printed = false;

        // Buffer for the assistant message (streamed in chunks) and any tool calls
        let mut assistant_buffer = String::new();
        // Keep track of tool calls we have already seen (by id) to avoid duplicate dispatches
        let mut message_tool_calls: Vec<_> = Vec::new();
        let mut seen_tool_call_ids: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        while let Some(event_res) = stream.next().await {
            match event_res {
                Ok(event) => match event {
                    ChatStreamEvent::Message(response) => {
                        if !assistant_prefix_printed {
                            print!("assistant: ");
                            assistant_prefix_printed = true;
                        }
                        // Print streaming chunk immediately
                        print!("{}", response.message.content);

                        // Accumulate into a single assistant buffer (avoid pushing partial messages to history)
                        assistant_buffer.push_str(&response.message.content);

                        // Collect any tool-calls emitted in this stream message (they may arrive in the final chunk)
                        if !response.message.tool_calls.is_empty() {
                            for tool_call in response.message.tool_calls.iter() {
                                // Deduplicate by tool call id so repeated chunks don't cause repeated dispatch
                                let is_new = seen_tool_call_ids.insert(tool_call.id.clone());
                                if is_new {
                                    message_tool_calls.push(tool_call.clone());
                                    let name = tool_call.function.name.clone();
                                    if name == "fibonacci" {
                                        if let Ok(n_value) =
                                            FibonacciTool::parse_n(&tool_call.function.arguments)
                                        {
                                            print!("[tool call: {}(n={} )]", name, n_value);
                                        } else {
                                            print!("[tool call: {}]", name);
                                        }
                                    } else {
                                        print!("[tool call: {}]", name);
                                    }
                                } else {
                                    // Duplicate tool call id received in another chunk; ignore.
                                }
                            }
                        }

                        // Only once the message is complete do we push it to history and handle tool calls.
                        if response.done {
                            history.push(ChatRequestMessage::Message(
                                RegularChatRequestMessage::new(
                                    Role::Assistant,
                                    assistant_buffer.clone(),
                                ),
                            ));

                            // If there are no tool calls, we're done.
                            if message_tool_calls.is_empty() {
                                println!("assistant: [done]");
                                return Ok(());
                            }

                            // Handle the first tool call in this message (subsequent calls can be handled in later iterations).
                            for call in message_tool_calls.iter() {
                                let tool_name = call.function.clone().name;
                                let tool_call_id = call.id.clone();
                                let params = call.function.arguments.clone();

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

                                            let tool_msg = ToolCallResultMessage::new(
                                                tool_name.clone(),
                                                serde_json::to_string(&tool_result)
                                                    .unwrap_or_else(|_| tool_result.to_string()),
                                                call.id.clone(),
                                            );
                                            history
                                                .push(ChatRequestMessage::ToolCallResult(tool_msg));

                                            tool_was_called = true;
                                            break;
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "tool_result: {} error: Tool invocation failed: {}",
                                                tool_name, e
                                            );

                                            let tool_msg = ToolCallResultMessage::new(
                                                tool_name.clone(),
                                                format!("Tool invocation error: {}", e),
                                                tool_call_id,
                                            );
                                            history
                                                .push(ChatRequestMessage::ToolCallResult(tool_msg));

                                            tool_was_called = true;
                                            break;
                                        }
                                    }
                                } else {
                                    eprintln!(
                                        "tool_result: {} error: No registered tool named '{}'",
                                        tool_name, tool_name
                                    );
                                    let tool_msg = ToolCallResultMessage::new(
                                        tool_name.clone(),
                                        format!("Tool '{}' not found", tool_name),
                                        tool_call_id,
                                    );
                                    history.push(ChatRequestMessage::ToolCallResult(tool_msg));

                                    tool_was_called = true;
                                    break;
                                }
                            }

                            if tool_was_called {
                                break;
                            }
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
