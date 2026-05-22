use crate::llm::{Message, LLMClient};
use anyhow::Result;

pub struct ContextManager {
    #[allow(dead_code)]
    max_tokens: usize,
    snip_at: usize,
    summarize_at: usize,
    collapse_at: usize,
}

impl ContextManager {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            snip_at: (max_tokens as f64 * 0.5) as usize,
            summarize_at: (max_tokens as f64 * 0.7) as usize,
            collapse_at: (max_tokens as f64 * 0.9) as usize,
        }
    }

    fn estimate_tokens(&self, messages: &[Message]) -> usize {
        messages.iter().map(|m| {
            let content_len = m.content.as_ref().map(|s| s.len()).unwrap_or(0);
            content_len / 4 // Very rough estimate
        }).sum()
    }

    pub async fn maybe_compress(&self, messages: &mut Vec<Message>, llm: &LLMClient) -> Result<bool> {
        let current = self.estimate_tokens(messages);
        let mut changed = false;

        if current > self.snip_at {
            changed |= self.snip_tool_outputs(messages);
        }

        if current > self.summarize_at && messages.len() > 10 {
            changed |= self.summarize_old(messages, llm).await?;
        }

        if current > self.collapse_at && messages.len() > 4 {
            self.hard_collapse(messages);
            changed = true;
        }

        Ok(changed)
    }

    fn snip_tool_outputs(&self, messages: &mut [Message]) -> bool {
        let mut changed = false;
        for m in messages.iter_mut() {
            if m.role == "tool" {
                if let Some(content) = &m.content {
                    if content.len() > 2000 {
                        let lines: Vec<&str> = content.lines().collect();
                        if lines.len() > 10 {
                            let snipped = format!(
                                "{}\n... ({} lines snipped) ...\n{}",
                                lines[..3].join("\n"),
                                lines.len() - 6,
                                lines[lines.len()-3..].join("\n")
                            );
                            m.content = Some(snipped);
                            changed = true;
                        }
                    }
                }
            }
        }
        changed
    }

    async fn summarize_old(&self, messages: &mut Vec<Message>, llm: &LLMClient) -> Result<bool> {
        // Preserve system message if it's the first one
        let has_system = !messages.is_empty() && messages[0].role == "system";
        let start_idx = if has_system { 1 } else { 0 };

        // Keep last 8 messages
        let split_idx = messages.len().saturating_sub(8);
        if split_idx <= start_idx { return Ok(false); }

        let old_messages = messages.drain(start_idx..split_idx).collect::<Vec<_>>();
        let summary_prompt = "Summarize the following conversation history briefly, \
                              preserving key decisions and edited files.";

        let mut summary_messages = old_messages;
        summary_messages.push(Message {
            role: "user".to_string(),
            content: Some(summary_prompt.to_string()),
            tool_calls: None,
            tool_call_id: None,
        });

        let resp = llm.chat(summary_messages, None).await?;
        let summary = resp.content.unwrap_or_else(|| "No summary generated".to_string());

        messages.insert(start_idx, Message {
            role: "user".to_string(),
            content: Some(format!("[Context Summary]\n{}", summary)),
            tool_calls: None,
            tool_call_id: None,
        });
        messages.insert(start_idx + 1, Message {
            role: "assistant".to_string(),
            content: Some("I have summarized the previous context.".to_string()),
            tool_calls: None,
            tool_call_id: None,
        });

        Ok(true)
    }

    fn hard_collapse(&self, messages: &mut Vec<Message>) {
        let has_system = !messages.is_empty() && messages[0].role == "system";
        let start_idx = if has_system { 1 } else { 0 };
        let keep = 4;

        if messages.len() > keep + start_idx {
            messages.drain(start_idx..messages.len() - keep);
            messages.insert(start_idx, Message {
                role: "user".to_string(),
                content: Some("[Hard Context Collapse - keeping only most recent]".to_string()),
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }
}


