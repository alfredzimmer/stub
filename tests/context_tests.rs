use stub::context::ContextManager;
use stub::llm::{Message, LLMClient};

fn test_llm() -> LLMClient {
    LLMClient::new(
        "test-model".to_string(),
        "test-key".to_string(),
        Some("http://localhost".to_string()),
    )
}

#[tokio::test]
async fn test_maybe_compress_no_change_for_small_messages() {
    let cm = ContextManager::new(1000);
    let mut messages = vec![
        Message { role: "user".to_string(), content: Some("hello".to_string()), tool_calls: None, tool_call_id: None },
        Message { role: "assistant".to_string(), content: Some("hi there".to_string()), tool_calls: None, tool_call_id: None },
    ];

    let changed = cm.maybe_compress(&mut messages, &test_llm()).await.unwrap();

    assert!(!changed);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].content.as_deref(), Some("hello"));
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(messages[1].content.as_deref(), Some("hi there"));
}

#[tokio::test]
async fn test_maybe_compress_snips_tool_output() {
    let cm = ContextManager::new(1000);
    let long_content = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11\n".repeat(40);
    let mut messages = vec![
        Message { role: "tool".to_string(), content: Some(long_content), tool_calls: None, tool_call_id: None },
    ];

    let changed = cm.maybe_compress(&mut messages, &test_llm()).await.unwrap();

    assert!(changed);
    let content = messages[0].content.as_ref().unwrap();
    assert!(content.contains("lines snipped"));
    assert!(content.starts_with("line1\nline2\nline3"));
}

#[tokio::test]
async fn test_maybe_compress_hard_collapse() {
    let cm = ContextManager::new(10);
    let mut messages = vec![
        Message { role: "user".to_string(), content: Some("abcdefgh".to_string()), tool_calls: None, tool_call_id: None },
        Message { role: "assistant".to_string(), content: Some("ijklmnop".to_string()), tool_calls: None, tool_call_id: None },
        Message { role: "user".to_string(), content: Some("qrstuvwx".to_string()), tool_calls: None, tool_call_id: None },
        Message { role: "assistant".to_string(), content: Some("yzabcdef".to_string()), tool_calls: None, tool_call_id: None },
        Message { role: "user".to_string(), content: Some("ghijklmn".to_string()), tool_calls: None, tool_call_id: None },
        Message { role: "assistant".to_string(), content: Some("opqrstuv".to_string()), tool_calls: None, tool_call_id: None },
    ];

    let changed = cm.maybe_compress(&mut messages, &test_llm()).await.unwrap();

    assert!(changed);
    assert_eq!(messages.len(), 5);
    assert_eq!(messages[0].role, "user");
    assert!(messages[0].content.as_deref().unwrap().contains("Hard Context Collapse"));
}
