//! Compact example: streaming chat + tool call dispatch.
//! The logic is intentionally split into small helpers to keep the
//! example concise and readable.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;
use futures::StreamExt;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use ollama_sdk::{
    tools::{DynTool, Tool, ToolContext},
    types::{
        chat::{
            ChatRequestMessage, ChatStreamEvent, FunctionalTool, RegularChatRequestMessage,
            StreamingChatRequest, ToolCall, ToolCallResultMessage, ToolSpec,
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
                v.as_u64()
                    .or_else(|| {
                        v.as_i64()
                            .and_then(|i| if i >= 0 { Some(i as u64) } else { None })
                    })
                    .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
            })
            .ok_or_else(|| Error::Tool("Missing or invalid parameter 'n'".to_string()))
    }

    fn fibonacci(n: u64) -> u128 {
        let (mut a, mut b) = (0u128, 1u128);
        for _ in 0..n {
            let c = a + b;
            a = b;
            b = c;
        }
        a
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

/// Build a streaming request from model, history and tools.
fn build_request(
    model: &str,
    history: &[ChatRequestMessage],
    tools: Vec<ToolSpec>,
) -> StreamingChatRequest {
    let mut req = StreamingChatRequest::new(model.to_string()).tools(tools);
    for msg in history {
        req = req.add_message(msg.clone());
    }
    req
}

/// Run a single tool call and return a `ToolCallResultMessage`.
async fn dispatch_tool_call(
    call: &ToolCall,
    tools: &HashMap<String, DynTool>,
) -> ToolCallResultMessage {
    let name = call.function.name.clone();
    let id = call.id.clone();
    let params = call.function.arguments.clone();

    if let Some(tool) = tools.get(&name) {
        let ctx = ToolContext {
            cancellation_token: CancellationToken::new(),
        };
        match tool.call(params, ctx).await {
            Ok(res) => {
                println!("tool_result: {} -> {}", name, res);
                let content = serde_json::to_string(&res).unwrap_or_else(|_| res.to_string());
                ToolCallResultMessage::new(name, content, id)
            }
            Err(e) => {
                eprintln!("tool_result: {} error: {}", name, e);
                ToolCallResultMessage::new(name, format!("Tool invocation error: {}", e), id)
            }
        }
    } else {
        eprintln!("tool_result: {} error: not registered", name);
        ToolCallResultMessage::new(name.clone(), format!("Tool '{}' not found", name), id)
    }
}

/// Consume the stream for one assistant message. Returns `Ok(true)` if a tool
/// was called (and its result added to history), `Ok(false)` to finish.
async fn process_stream<S>(
    stream: &mut S,
    tools: &HashMap<String, DynTool>,
    history: &mut Vec<ChatRequestMessage>,
) -> std::result::Result<bool, Error>
where
    S: futures::Stream<Item = std::result::Result<ChatStreamEvent, Error>> + Unpin,
{
    let mut assistant_buffer = String::new();
    let mut seen_tool_call_ids = HashSet::new();
    let mut collected_calls: Vec<ToolCall> = Vec::new();
    let mut assistant_prefix_printed = false;

    while let Some(event_res) = stream.next().await {
        match event_res {
            Ok(ChatStreamEvent::Message(response)) => {
                if !assistant_prefix_printed {
                    print!("assistant: ");
                    assistant_prefix_printed = true;
                }

                // Stream the chunk and accumulate the assistant message.
                print!("{}", response.message.content);
                assistant_buffer.push_str(&response.message.content);

                // Record any tool calls (deduplicated by id).
                for call in &response.message.tool_calls {
                    if seen_tool_call_ids.insert(call.id.clone()) {
                        if call.function.name == "fibonacci" {
                            if let Ok(n) = FibonacciTool::parse_n(&call.function.arguments) {
                                print!("[tool call: {}(n={})]", call.function.name, n);
                            } else {
                                print!("[tool call: {}]", call.function.name);
                            }
                        } else {
                            print!("[tool call: {}]", call.function.name);
                        }
                        collected_calls.push(call.clone());
                    }
                }

                // Once the message is complete, either finish or dispatch the first tool.
                if response.done {
                    println!();
                    history.push(ChatRequestMessage::Message(RegularChatRequestMessage::new(
                        Role::Assistant,
                        assistant_buffer.clone(),
                    )));

                    if collected_calls.is_empty() {
                        println!("assistant: [done]");
                        return Ok(false);
                    }

                    let result_msg = dispatch_tool_call(&collected_calls[0], tools).await;
                    history.push(ChatRequestMessage::ToolCallResult(result_msg));
                    return Ok(true);
                }
            }
            Ok(ChatStreamEvent::Error(err)) => {
                eprintln!("server error chunk: {}", err);
            }
            Ok(ChatStreamEvent::Partial { partial, error }) => {
                eprintln!("partial response: {} {:?}", partial, error);
            }
            Err(e) => {
                eprintln!("streaming error: {}", e);
            }
        }
    }

    Ok(false)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Client + tool registration.
    let mut client = OllamaClient::builder().build()?;
    let fib_tool: DynTool = Arc::new(FibonacciTool);
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
                "n": { "type": "integer", "minimum": 0 }
            },
            "required": ["n"]
        }),
    };

    let user_prompt = "Use the `fibonacci` function to compute fibonacci(n) and return the value. \
        For example, compute fibonacci for n=31 and n=11. Make sure to call the function \
        instead of trying to compute it yourself or making up values."
        .to_string();

    let mut history = vec![
        RegularChatRequestMessage::new(Role::User, user_prompt.clone()).to_chat_request_message(),
    ];
    let tools = vec![ToolSpec::Function {
        function: fib_tool_spec.clone(),
    }];

    println!("user: {}", user_prompt);

    let mut tool_map: HashMap<String, DynTool> = HashMap::new();
    tool_map.insert(fib_tool.name().to_string(), fib_tool.clone());

    // Keep sending updated history until no tool is invoked.
    loop {
        let request = build_request(&model, &history, tools.clone());
        let mut stream = client.chat_stream(request).await?;
        let tool_was_called = process_stream(&mut stream, &tool_map, &mut history).await?;
        if !tool_was_called {
            break;
        }
    }

    Ok(())
}
