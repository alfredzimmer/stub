use stub::tools::edit::EditFileTool;
use stub::tools::Tool;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_edit_file_success() {
    let tmpfile = NamedTempFile::new().unwrap();
    let path = tmpfile.path().to_str().unwrap().to_string();
    
    {
        let mut file = tokio::fs::File::create(&path).await.unwrap();
        file.write_all(b"line1\nline2\nline3\n").await.unwrap();
    }

    let tool = EditFileTool;
    let args = json!({
        "file_path": path,
        "old_string": "line2",
        "new_string": "line2 modified"
    });

    let result = tool.execute(args).await.unwrap();
    assert!(result.contains("Successfully edited"));
    assert!(result.contains(&format!("--- a/{}", path)));
    assert!(result.contains(&format!("+++ b/{}", path)));
    assert!(result.contains("-line2"));
    assert!(result.contains("+line2 modified"));

    let content = tokio::fs::read_to_string(path).await.unwrap();
    assert_eq!(content, "line1\nline2 modified\nline3\n");
}

#[tokio::test]
async fn test_edit_file_not_found() {
    let tmpfile = NamedTempFile::new().unwrap();
    let path = tmpfile.path().to_str().unwrap().to_string();
    
    {
        let mut file = tokio::fs::File::create(&path).await.unwrap();
        file.write_all(b"line1\nline2\nline3\n").await.unwrap();
    }

    let tool = EditFileTool;
    let args = json!({
        "file_path": path,
        "old_string": "missing",
        "new_string": "replacement"
    });

    let result = tool.execute(args).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Ensure the string matches exactly"));
}

#[tokio::test]
async fn test_edit_file_multiple_occurrences() {
    let tmpfile = NamedTempFile::new().unwrap();
    let path = tmpfile.path().to_str().unwrap().to_string();
    
    {
        let mut file = tokio::fs::File::create(&path).await.unwrap();
        file.write_all(b"dup\ndup\n").await.unwrap();
    }

    let tool = EditFileTool;
    let args = json!({
        "file_path": path,
        "old_string": "dup",
        "new_string": "single"
    });

    let result = tool.execute(args).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("appears 2 times"));
}

#[tokio::test]
async fn test_edit_file_empty_old_string() {
    let tool = EditFileTool;
    let args = json!({
        "file_path": "any.txt",
        "old_string": "",
        "new_string": "something"
    });

    let result = tool.execute(args).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "old_string cannot be empty");
}
