use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;

pub mod file;
pub mod edit;
pub mod bash;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn parameters(&self) -> Value;
    async fn execute(&self, args: Value) -> Result<String>;
    
    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name(),
                "description": self.description(),
                "parameters": self.parameters(),
            }
        })
    }
}

pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }
    pub fn add(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }
    pub fn get_schemas(&self) -> Vec<Value> {
        self.tools.iter().map(|t| t.schema()).collect()
    }
    pub async fn call(&self, name: &str, args: Value) -> Result<String> {
        for t in &self.tools {
            if t.name() == name {
                return t.execute(args).await;
            }
        }
        anyhow::bail!("Tool not found: {}", name)
    }
}
