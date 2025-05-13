use colored::*;
use regex::Regex;
use shellai::OpenAIAgent;
use std::io::{self, Write};
use std::process::Command;

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

    // Compile regex patterns for code blocks
    // This pattern matches ```bash, ```sh, or just ``` followed by content that looks like bash
    let bash_regex = Regex::new(r"```(?:bash|sh|)([\s\S]*?)```").unwrap();

    // Interactive loop
    loop {
        // Print prompt
        print!("\n{}: ", "You".bright_green());
        io::stdout().flush()?;

        // Read user input
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;

        // Trim whitespace
        let user_input = user_input.trim();

        // Check for exit command
        if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
            println!("{}", "Goodbye!".bright_blue());
            break;
        }

        // Skip empty inputs
        if user_input.is_empty() {
            continue;
        }

        // Show thinking indicator
        print!("{}", "\nAI is thinking...".bright_yellow());
        io::stdout().flush()?;

        // Get response from OpenAI
        match agent.generate_response(user_input).await {
            Ok(response) => {
                // Clear the "thinking" indicator
                print!("\r{}", " ".repeat(16));
                print!("\r");
                // Print the response
                println!("{}: {}", "AI".bright_blue(), response);

                // Check if the response contains bash code
                let bash_blocks: Vec<_> = bash_regex.captures_iter(&response).collect();

                // If bash code is found, ask if the user wants to execute it
                if !bash_blocks.is_empty() {
                    for (i, capture) in bash_blocks.iter().enumerate() {
                        if let Some(code) = capture.get(1) {
                            let bash_code = code.as_str().trim();

                            println!(
                                "\n{} #{}",
                                "Bash code block".bright_yellow(),
                                (i + 1).to_string().bright_yellow()
                            );
                            println!(
                                "{}",
                                "┌─────────────────────────────────────────────┐".bright_red()
                            );

                            // Split the code into lines and print each with proper formatting
                            for line in bash_code.lines() {
                                println!("{} {}", "│".bright_red(), line.bright_white().on_black());
                            }

                            println!(
                                "{}",
                                "└─────────────────────────────────────────────┘".bright_red()
                            );

                            print!(
                                "{} (y/n): ",
                                "Do you want to execute this code?".bright_yellow()
                            );
                            io::stdout().flush()?;

                            let mut execute_input = String::new();
                            io::stdin().read_line(&mut execute_input)?;

                            if execute_input.trim().eq_ignore_ascii_case("y") {
                                println!("{}", "Executing bash code...".bright_green());

                                // Execute the bash code
                                let output =
                                    Command::new("bash").arg("-c").arg(bash_code).output()?;

                                // Print the command output
                                if !output.stdout.is_empty() {
                                    println!("{}", "Output:".bright_green());
                                    println!("{}", String::from_utf8_lossy(&output.stdout));
                                }

                                // Print any errors
                                if !output.stderr.is_empty() {
                                    println!("{}", "Errors:".bright_red());
                                    println!(
                                        "{}",
                                        String::from_utf8_lossy(&output.stderr).bright_red()
                                    );
                                }

                                let status_str =
                                    format!("Execution completed with status: {}", output.status);
                                if output.status.success() {
                                    println!("{}", status_str.bright_green());
                                } else {
                                    println!("{}", status_str.bright_red());
                                }
                            } else {
                                println!("{}", "Code execution skipped.".bright_yellow());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // Clear the "thinking" indicator
                print!("\r{}", " ".repeat(16));
                print!("\r");
                eprintln!("{}: {}", "Error".bright_red(), e);
            }
        }
    }

    Ok(())
}
