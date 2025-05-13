use shellai::OpenAIAgent;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ShellAI - Your AI assistant in the terminal");
    println!("Type 'exit' or 'quit' to end the session");

    // Create an OpenAI agent
    let agent = match OpenAIAgent::new("gpt-4".to_string()) {
        Ok(agent) => agent,
        Err(e) => {
            eprintln!("Error initializing OpenAI agent: {}", e);
            eprintln!("Make sure the OPENAI_API_KEY environment variable is set.");
            return Err(e);
        }
    };

    // Interactive loop
    loop {
        // Print prompt
        print!("\nYou: ");
        io::stdout().flush()?;

        // Read user input
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;

        // Trim whitespace
        let user_input = user_input.trim();

        // Check for exit command
        if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
            println!("Goodbye!");
            break;
        }

        // Skip empty inputs
        if user_input.is_empty() {
            continue;
        }

        // Show thinking indicator
        print!("AI is thinking");
        io::stdout().flush()?;

        // Get response from OpenAI
        match agent.generate_response(user_input).await {
            Ok(response) => {
                // Clear the "thinking" indicator
                print!("\r");
                // Print the response
                println!("AI: {}", response);
            }
            Err(e) => {
                // Clear the "thinking" indicator
                print!("\r");
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
