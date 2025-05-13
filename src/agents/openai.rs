// OpenAI Agent Implementation

use anyhow::anyhow;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

// Default system prompt as fallback if directory scanning fails
const DEFAULT_SYSTEM_PROMPT: &str = r#"You are ShellAI, a helpful AI assistant in a terminal environment.

When responding, follow these guidelines:

1. When providing bash commands or scripts, always format them in code blocks using ```bash and ``` syntax.
2. Prefer providing executable bash commands when appropriate for the user's request.
3. Keep your bash commands clear, concise, and safe to execute.
4. Include comments in your bash code to explain what each command or section does.
5. For complex operations, break them down into smaller, manageable commands.
6. Always explain what your bash commands will do before showing the code.
7. After showing bash code, explain the expected output or result.
8. If a command might have system-altering effects (like deleting files), provide clear warnings.
9. When possible, include error handling in your bash scripts.
10. Format your responses clearly with appropriate spacing and organization.

Remember that the user can execute your bash code directly from the terminal interface, so make sure your commands are correct and safe."#;

#[derive(Debug)]
pub struct OpenAIAgent {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

// Function to scan directory and build a tree structure
fn scan_directory(path: &Path, max_depth: usize, current_depth: usize) -> Result<String, Box<dyn Error>> {
    if current_depth > max_depth {
        return Ok("...".to_string());
    }

    let mut result = String::new();
    
    if path.is_dir() {
        let entries = fs::read_dir(path)?;
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            
            // Skip hidden files and directories
            if file_name.starts_with('.') {
                continue;
            }
            
            if path.is_dir() {
                dirs.push(file_name);
            } else {
                files.push(file_name);
            }
        }
        
        // Sort directories and files for consistent output
        dirs.sort();
        files.sort();
        
        // Add directories first
        for dir in dirs {
            let indent = "  ".repeat(current_depth);
            result.push_str(&format!("{}📁 {}/\n", indent, dir));
            
            let subdir_path = path.join(&dir);
            let subdir_content = scan_directory(&subdir_path, max_depth, current_depth + 1)?;
            result.push_str(&subdir_content);
        }
        
        // Then add files
        for file in files {
            let indent = "  ".repeat(current_depth);
            result.push_str(&format!("{}📄 {}\n", indent, file));
        }
    }
    
    Ok(result)
}

// Function to get the current working directory
fn get_current_directory() -> Result<PathBuf, Box<dyn Error>> {
    let current_dir = env::current_dir()?;
    Ok(current_dir)
}

// Function to build the system prompt with directory information
fn build_system_prompt() -> Result<String, Box<dyn Error>> {
    let current_dir = get_current_directory()?;
    let dir_name = current_dir.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    let dir_path = current_dir.to_string_lossy().to_string();
    
    // Scan the directory structure (limit depth to 2 to avoid overwhelming output)
    let dir_tree = scan_directory(&current_dir, 2, 0)?;
    
    let prompt = format!(r#"You are ShellAI, a helpful AI assistant in a terminal environment.

Current working directory: {}
Directory name: {}

Directory structure:
{}

Important: The user is using a terminal interface where they can press Enter to create new lines within their question. Treat all lines as part of a single coherent question or request, even if they appear to be separate statements. The user may be formatting their question across multiple lines for clarity.

When responding, follow these guidelines:

1. When providing bash commands or scripts, always format them in code blocks using ```bash and ``` syntax.
2. Prefer providing executable bash commands when appropriate for the user's request.
3. Keep your bash commands clear, concise, and safe to execute.
4. Include comments in your bash code to explain what each command or section does.
5. For complex operations, break them down into smaller, manageable commands.
6. Always explain what your bash commands will do before showing the code.
7. After showing bash code, explain the expected output or result.
8. If a command might have system-altering effects (like deleting files), provide clear warnings.
9. When possible, include error handling in your bash scripts.
10. Format your responses clearly with appropriate spacing and organization.
11. Be aware of the current directory structure shown above when suggesting commands.
12. When referencing files or directories, use the correct paths based on the current directory.

Remember that the user can execute your bash code directly from the terminal interface, so make sure your commands are correct and safe."#, dir_path, dir_name, dir_tree);

    Ok(prompt)
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatMessage,
}

impl OpenAIAgent {
    pub fn new(model: String) -> Result<Self, Box<dyn Error>> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable not set")?;

        let client = reqwest::Client::new();

        Ok(Self {
            api_key,
            model,
            client,
        })
    }

    pub async fn generate_response(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        // Create headers with authorization
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Get the dynamic system prompt with directory information
        let system_prompt = match build_system_prompt() {
            Ok(prompt) => prompt,
            Err(e) => {
                eprintln!("Warning: Failed to build dynamic system prompt: {}", e);
                DEFAULT_SYSTEM_PROMPT.to_string()
            }
        };

        // Create the request body with system prompt and user message
        let request_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.7,
        };

        // Make the API request
        let response = self
            .client
            .post(OPENAI_API_URL)
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        // Check if the request was successful
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("API request failed: {}", error_text).into());
        }

        // Parse the response
        let completion: ChatCompletionResponse = response.json().await?;

        // Extract the response text
        if let Some(choice) = completion.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow!("No response from API").into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    // Save the original environment variable value before tests and restore it after
    fn with_env_var<F>(key: &str, value: Option<&str>, test: F)
    where
        F: FnOnce(),
    {
        let original = env::var(key).ok();

        match value {
            Some(val) => env::set_var(key, val),
            None => env::remove_var(key),
        }

        // Run the test
        test();

        // Restore the original value
        match original {
            Some(val) => env::set_var(key, val),
            None => env::remove_var(key),
        }
    }

    #[test]
    fn test_openai_agent_creation() {
        with_env_var("OPENAI_API_KEY", Some("test_key"), || {
            let agent = OpenAIAgent::new("gpt-4".to_string()).expect("Failed to create agent");
            assert_eq!(agent.api_key, "test_key");
            assert_eq!(agent.model, "gpt-4");
            // We can't directly test the client, but we can verify it was created
        });
    }

    #[test]
    fn test_openai_agent_creation_error() {
        with_env_var("OPENAI_API_KEY", None, || {
            let result = OpenAIAgent::new("gpt-4".to_string());
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("OPENAI_API_KEY"));
        });
    }

    #[test]
    fn test_scan_directory() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();
        
        // Create a test directory structure
        let subdir1 = temp_path.join("subdir1");
        let subdir2 = temp_path.join("subdir2");
        let nested_dir = subdir1.join("nested");
        
        fs::create_dir(&subdir1).expect("Failed to create subdir1");
        fs::create_dir(&subdir2).expect("Failed to create subdir2");
        fs::create_dir(&nested_dir).expect("Failed to create nested dir");
        
        // Create some test files
        let file1 = temp_path.join("file1.txt");
        let file2 = subdir1.join("file2.txt");
        let file3 = nested_dir.join("file3.txt");
        
        File::create(&file1).and_then(|mut f| f.write_all(b"test content")).expect("Failed to create file1");
        File::create(&file2).and_then(|mut f| f.write_all(b"test content")).expect("Failed to create file2");
        File::create(&file3).and_then(|mut f| f.write_all(b"test content")).expect("Failed to create file3");
        
        // Create a hidden file and directory (should be skipped)
        let hidden_file = temp_path.join(".hidden_file");
        let hidden_dir = temp_path.join(".hidden_dir");
        
        fs::create_dir(&hidden_dir).expect("Failed to create hidden dir");
        File::create(&hidden_file).and_then(|mut f| f.write_all(b"hidden content")).expect("Failed to create hidden file");
        
        // Test scanning with max_depth = 2
        let result = scan_directory(temp_path, 2, 0).expect("Failed to scan directory");
        
        // Verify the result contains expected entries
        assert!(result.contains("📁 subdir1/"));
        assert!(result.contains("📁 subdir2/"));
        assert!(result.contains("📄 file1.txt"));
        assert!(result.contains("📁 nested/"));
        assert!(result.contains("📄 file2.txt"));
        
        // Verify hidden files/dirs are not included
        assert!(!result.contains(".hidden_file"));
        assert!(!result.contains(".hidden_dir"));
        
        // Test with max_depth = 0 (should only show top-level directories and files)
        let limited_result = scan_directory(temp_path, 0, 0).expect("Failed to scan directory with limit");
        assert!(limited_result.contains("📁 subdir1/"));
        assert!(limited_result.contains("..."));
        assert!(!limited_result.contains("📄 file2.txt"));
    }

    #[test]
    fn test_build_system_prompt() {
        // This is a basic test to ensure the function runs without errors
        // We can't easily test the exact content since it depends on the current directory
        let result = build_system_prompt();
        assert!(result.is_ok());
        
        let prompt = result.unwrap();
        assert!(prompt.contains("Current working directory:"));
        assert!(prompt.contains("Directory name:"));
        assert!(prompt.contains("Directory structure:"));
    }

    // Mock test for generate_response would require more complex setup with HTTP mocking
    // libraries like mockito or wiremock, which we'll omit for simplicity
}
