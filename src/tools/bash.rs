use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;
use anyhow::{Result, anyhow};
use tokio::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;
use regex::Regex;

pub struct BashTool {
    cwd: Arc<Mutex<PathBuf>>,
}

impl BashTool {
    pub fn new(initial_cwd: PathBuf) -> Self {
        Self { cwd: Arc::new(Mutex::new(initial_cwd)) }
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> String { "bash".to_string() }
    fn description(&self) -> String {
        "Execute a shell command. Returns stdout, stderr, and exit code. \
         Use this for running tests, installing packages, git operations, etc.".to_string()
    }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string" }
            },
            "required": ["command"]
        })
    }
    async fn execute(&self, args: Value) -> Result<String> {
        let command = args["command"].as_str().ok_or_else(|| anyhow!("Missing command"))?;
        
        // Improved security check using regex
        let dangerous_patterns = [
            r"rm\s+-rf\s+/",
            r"rm\s+-rf\s+\*",
            r"mkfs",
            r"dd\s+if=",
            r":\(\)\{ :\|:& \};:",
        ];
        for pattern in dangerous_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(command) {
                    return Ok(format!("⚠ Blocked dangerous command pattern: {}", pattern));
                }
            }
        }

        let mut cwd = self.cwd.lock().await;

        // Robust CD tracking: wrap the command to output the final PWD.
        // We use a sentinel to reliably extract it from stdout.
        let wrapped_command = format!("{}; printf '\\n__PWD__:%s\\n' \"$(pwd -P)\"", command);

        let output = Command::new("sh")
            .arg("-c")
            .arg(&wrapped_command)
            .current_dir(&*cwd)
            .output()
            .await?;

        let full_stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let status = output.status.code().unwrap_or(-1);

        // Parse PWD and clean up stdout
        let mut stdout_lines = Vec::new();
        let mut new_pwd = None;

        for line in full_stdout.lines() {
            if let Some(stripped) = line.strip_prefix("__PWD__:") {
                new_pwd = Some(stripped.to_string());
            } else {
                stdout_lines.push(line);
            }
        }

        if output.status.success() {
            if let Some(pwd_str) = new_pwd {
                let new_path = PathBuf::from(pwd_str);
                if new_path.is_dir() {
                    // Canonicalize to ensure absolute path and resolve symlinks
                    if let Ok(canonical) = std::fs::canonicalize(new_path) {
                        *cwd = canonical;
                    }
                }
            }
        }

        let stdout = stdout_lines.join("\n");
        let mut result = stdout.trim().to_string();
        if !stderr.is_empty() {
            if !result.is_empty() { result.push('\n'); }
            result.push_str(&format!("[stderr]\n{}", stderr.trim()));
        }
        if status != 0 {
            if !result.is_empty() { result.push('\n'); }
            result.push_str(&format!("[exit code: {}]", status));
        }

        Ok(result.trim().to_string())
    }
}
