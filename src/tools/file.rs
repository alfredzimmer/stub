use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;
use tokio::fs;

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> String { "read_file".to_string() }
    fn description(&self) -> String { "Read the content of a file.".to_string() }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string" }
            },
            "required": ["file_path"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String> {
        let path = args["file_path"].as_str().ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
        let content = fs::read_to_string(path).await?;
        Ok(content)
    }
}

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> String { "write_file".to_string() }
    fn description(&self) -> String { "Write content to a file.".to_string() }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["file_path", "content"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String> {
        let path = args["file_path"].as_str().ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
        let content = args["content"].as_str().ok_or_else(|| anyhow::anyhow!("Missing content"))?;
        fs::write(path, content).await?;
        Ok(format!("Successfully wrote to {}", path))
    }
}

pub struct GlobTool;

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> String { "glob".to_string() }
    fn description(&self) -> String { "Search for files matching a glob pattern.".to_string() }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string" }
            },
            "required": ["pattern"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String> {
        let pattern = args["pattern"].as_str().ok_or_else(|| anyhow::anyhow!("Missing pattern"))?;
        let mut matches = Vec::new();
        // glob crate is sync, so we run it in a blocking task if needed, but for simplicity:
        for entry in glob::glob(pattern)? {
            match entry {
                Ok(path) => matches.push(path.to_string_lossy().to_string()),
                Err(e) => return Err(e.into()),
            }
        }
        Ok(matches.join("\n"))
    }
}

pub struct GrepTool;

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> String { "grep_search".to_string() }
    fn description(&self) -> String { "Search for a pattern in files.".to_string() }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string" },
                "path": { "type": "string" },
                "recursive": { "type": "boolean" }
            },
            "required": ["pattern", "path"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String> {
        let pattern_str = args["pattern"].as_str().ok_or_else(|| anyhow::anyhow!("Missing pattern"))?;
        let path = args["path"].as_str().ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        let recursive = args["recursive"].as_bool().unwrap_or(false);
        
        let re = regex::Regex::new(pattern_str)?;
        let mut results = Vec::new();

        if recursive {
            for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    if let Ok(content) = fs::read_to_string(file_path).await {
                        for (i, line) in content.lines().enumerate() {
                            if re.is_match(line) {
                                results.push(format!("{}:{}: {}", file_path.display(), i + 1, line));
                            }
                        }
                    }
                }
            }
        } else {
            let content = fs::read_to_string(path).await?;
            for (i, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    results.push(format!("{}:{}: {}", path, i + 1, line));
                }
            }
        }

        Ok(results.join("\n"))
    }
}


