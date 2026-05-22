use stub::llm::Message;

#[test]
fn test_message_serialization() {
    let msg = Message {
        role: "user".to_string(),
        content: Some("hello".to_string()),
        tool_calls: None,
        tool_call_id: None,
    };
    let serialized = serde_json::to_string(&msg).unwrap();
    assert_eq!(serialized, r#"{"role":"user","content":"hello"}"#);

    let deserialized: Message = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.role, "user");
    assert_eq!(deserialized.content, Some("hello".to_string()));
}
