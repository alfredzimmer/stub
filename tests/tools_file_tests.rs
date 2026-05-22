use stub::tools::file::{ReadFileTool, WriteFileTool, GlobTool, GrepTool};
use stub::tools::Tool;
use tempfile::tempdir;

#[tokio::test]
async fn test_file_tools() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let file_path = dir.path().join("test.txt");
    let file_path_str = file_path.to_str().unwrap();

    // Test WriteFileTool
    let write_tool = WriteFileTool;
    let write_args = serde_json::json!({
        "file_path": file_path_str,
        "content": "hello world\nline 2"
    });
    write_tool.execute(write_args).await?;

    // Test ReadFileTool
    let read_tool = ReadFileTool;
    let read_args = serde_json::json!({
        "file_path": file_path_str
    });
    let content = read_tool.execute(read_args).await?;
    assert_eq!(content, "hello world\nline 2");

    // Test GrepTool
    let grep_tool = GrepTool;
    let grep_args = serde_json::json!({
        "pattern": "world",
        "path": file_path_str
    });
    let grep_res = grep_tool.execute(grep_args).await?;
    assert!(grep_res.contains("hello world"));

    // Test GlobTool
    let glob_tool = GlobTool;
    let glob_args = serde_json::json!({
        "pattern": file_path_str
    });
    let glob_res = glob_tool.execute(glob_args).await?;
    assert!(glob_res.contains("test.txt"));

    Ok(())
}
