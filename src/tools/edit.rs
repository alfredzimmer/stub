use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;
use anyhow::{Result, anyhow};
use tokio::fs;
use similar::{TextDiff};

pub struct EditFileTool;

#[async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> String { "edit_file".to_string() }
    fn description(&self) -> String {
        "Edit a file by replacing an exact string match. \
         old_string must appear exactly once in the file for safety. \
         Include enough surrounding context to ensure uniqueness.".to_string()
    }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string" },
                "old_string": { "type": "string" },
                "new_string": { "type": "string" }
            },
            "required": ["file_path", "old_string", "new_string"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String> {
        let path = args["file_path"].as_str().ok_or_else(|| anyhow!("Missing file_path"))?;
        let old_string = args["old_string"].as_str().ok_or_else(|| anyhow!("Missing old_string"))?;
        let new_string = args["new_string"].as_str().ok_or_else(|| anyhow!("Missing new_string"))?;

        if old_string.is_empty() {
            return Err(anyhow!("old_string cannot be empty"));
        }

        let content = fs::read_to_string(path).await?;
        let count = content.matches(old_string).count();

        if count == 0 {
            return Err(anyhow!("old_string not found in {}. Ensure the string matches exactly, including whitespace and indentation.", path));
        }
        if count > 1 {
            return Err(anyhow!("old_string appears {} times in {}. Include more context.", count, path));
        }

        let new_content = content.replacen(old_string, new_string, 1);
        fs::write(path, &new_content).await?;

        // Generate unified diff
        let diff = TextDiff::from_lines(&content, &new_content);
        let mut diff_str = String::new();
        let mut unified_diff = diff.unified_diff();
        let formatted_diff = unified_diff
            .context_radius(3)
            .header(&format!("a/{}", path), &format!("b/{}", path))
            .to_string();
        diff_str.push_str(&formatted_diff);

        Ok(format!("Successfully edited {}\n{}", path, diff_str))
    }
}


