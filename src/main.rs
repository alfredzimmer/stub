use stub::llm::LLMClient;
use stub::tools::{ToolRegistry, file, edit, bash};
use stub::context::ContextManager;
use stub::agent::Agent;
use inquire::Text;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
    let base_url = env::var("OPENAI_BASE_URL").ok();

    let llm = LLMClient::new(model, api_key, base_url);
    let mut registry = ToolRegistry::new();
    
    registry.add(Box::new(file::ReadFileTool));
    registry.add(Box::new(file::WriteFileTool));
    registry.add(Box::new(file::GlobTool));
    registry.add(Box::new(file::GrepTool));
    registry.add(Box::new(edit::EditFileTool));
    
    let initial_cwd = env::current_dir()?;
    registry.add(Box::new(bash::BashTool::new(initial_cwd)));

    let context = ContextManager::new(128_000);
    let mut agent = Agent::new(llm, registry, context);

    println!("Stub: Minimal Rust Coding Agent. Type 'exit' to quit.");

    loop {
        let input = Text::new(">").prompt();
        match input {
            Ok(content) => {
                if content == "exit" || content == "quit" {
                    break;
                }
                if content.trim().is_empty() {
                    continue;
                }
                match agent.chat(&content).await {
                    Ok(response) => println!("\n{}", response),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}
