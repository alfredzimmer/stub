use crate::llm::{LLMClient, Message};
use crate::tools::ToolRegistry;
use crate::context::ContextManager;
use crate::prompt::get_system_prompt;
use anyhow::Result;
use std::sync::Arc;
use futures::future::join_all;

pub struct Agent {
    llm: LLMClient,
    registry: Arc<ToolRegistry>,
    context: ContextManager,
    messages: Vec<Message>,
    max_iterations: usize,
}

impl Agent {
    pub fn new(llm: LLMClient, registry: ToolRegistry, context: ContextManager) -> Self {
        let mut messages = Vec::new();
        messages.push(Message {
            role: "system".to_string(),
            content: Some(get_system_prompt()),
            tool_calls: None,
            tool_call_id: None,
        });

        Self {
            llm,
            registry: Arc::new(registry),
            context,
            messages,
            max_iterations: 30,
        }
    }

    pub async fn chat(&mut self, input: &str) -> Result<String> {
        self.messages.push(Message {
            role: "user".to_string(),
            content: Some(input.to_string()),
            tool_calls: None,
            tool_call_id: None,
        });

        for _ in 0..self.max_iterations {
            // Compress if needed
            self.context.maybe_compress(&mut self.messages, &self.llm).await?;

            let schemas = self.registry.get_schemas();
            let response = self.llm.chat(self.messages.clone(), Some(schemas)).await?;
            
            self.messages.push(response.clone());

            if let Some(tool_calls) = response.tool_calls {
                if tool_calls.is_empty() {
                    if let Some(content) = response.content {
                        return Ok(content);
                    }
                    // If no tool calls and no content, continue looping (might be a thought)
                    continue;
                }

                // Execute tool calls in parallel
                let mut futures = Vec::new();
                for tc in tool_calls {
                    println!("  > {}({})", tc.function.name, tc.function.arguments);
                    let registry = Arc::clone(&self.registry);
                    futures.push(async move {
                        let args_res: Result<serde_json::Value> = serde_json::from_str(&tc.function.arguments).map_err(|e| e.into());
                        let result = match args_res {
                            Ok(args) => match registry.call(&tc.function.name, args).await {
                                Ok(res) => res,
                                Err(e) => format!("Error executing {}: {}", tc.function.name, e),
                            },
                            Err(e) => format!("Error parsing arguments for {}: {}", tc.function.name, e),
                        };
                        Message {
                            role: "tool".to_string(),
                            content: Some(result),
                            tool_calls: None,
                            tool_call_id: Some(tc.id),
                        }
                    });
                }

                let results = join_all(futures).await;
                self.messages.extend(results);
            } else if let Some(content) = response.content {
                return Ok(content);
            } else {
                // No tool calls and no content? Break to avoid infinite loop.
                break;
            }
        }

        anyhow::bail!("Reached maximum iterations ({})", self.max_iterations)
    }
}
