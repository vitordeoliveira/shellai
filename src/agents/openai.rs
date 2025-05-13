// OpenAI Agent Implementation

use anyhow::anyhow;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

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

// System prompt to guide the AI's responses
const SYSTEM_PROMPT: &str = r#"You are ShellAI, a helpful AI assistant in a terminal environment. 

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

Remember that the user can execute your bash code directly from the terminal interface, so make sure your commands are correct and safe."#;

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

        // Create the request body with system prompt and user message
        let request_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: SYSTEM_PROMPT.to_string(),
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

    // Mock test for generate_response would require more complex setup with HTTP mocking
    // libraries like mockito or wiremock, which we'll omit for simplicity
}
