use colored::*;
use crossterm::{
    cursor::{MoveToColumn, MoveUp},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use regex::Regex;
use shellai::OpenAIAgent;
use std::io::{self, Write};
use std::process::Command;

/// Read multiline input from the user, with Enter adding a new line and Ctrl+S submitting
fn read_multiline_input() -> Result<String, Box<dyn std::error::Error>> {
    let mut buffer = String::new();

    // Enable raw mode to capture key events
    enable_raw_mode()?;

    // Print initial prompt
    print!(""); // Ensure cursor is at the right position
    io::stdout().flush()?;

    loop {
        // Wait for a key event
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match code {
                // Ctrl+S to submit
                KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
                    disable_raw_mode()?;
                    println!(); // Move to next line after submission
                    break;
                }

                // Enter key adds a newline character
                KeyCode::Enter => {
                    buffer.push('\n');
                    println!();
                    io::stdout().flush()?;
                }

                // Backspace key
                KeyCode::Backspace => {
                    if !buffer.is_empty() {
                        // Remove the last character
                        if buffer.ends_with('\n') {
                            // If we're at the start of a line, move up
                            buffer.pop();
                            execute!(io::stdout(), MoveUp(1), MoveToColumn(0))?;

                            // Find the length of the previous line
                            let last_line_len = buffer.lines().last().map_or(0, |line| line.len());

                            // Move to the end of the previous line
                            execute!(io::stdout(), MoveToColumn(last_line_len as u16))?;
                        } else {
                            buffer.pop();
                            // Move cursor back and erase the character
                            print!("\x08 \x08");
                            io::stdout().flush()?;
                        }
                    }
                }

                // Regular character input
                KeyCode::Char(c) => {
                    // Handle Ctrl+C to exit
                    if c == 'c' && modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        return Ok("exit".to_string());
                    }

                    buffer.push(c);
                    print!("{}", c);
                    io::stdout().flush()?;
                }

                // Escape key to cancel
                KeyCode::Esc => {
                    disable_raw_mode()?;
                    return Ok("".to_string());
                }

                _ => {}
            }
        }
    }

    Ok(buffer)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ShellAI - Your AI assistant in the terminal");

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
        // Print help text near the prompt area
        println!("\n{}", "─".repeat(60).bright_black());
        println!("{}", "ShellAI Commands:".bright_yellow());
        println!("{} - Add a new line", "Enter".bright_cyan());
        println!("{} - Submit your question", "Ctrl+S".bright_cyan());
        println!(
            "{} - Exit the application",
            "Ctrl+C or type 'exit'/'quit'".bright_cyan()
        );
        println!("{} - Cancel current input", "Esc".bright_cyan());
        println!("{} - Navigate and edit text", "Backspace".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());

        // Print prompt
        print!("{}: ", "You".bright_green());
        io::stdout().flush()?;

        // Read multiline user input
        let user_input = read_multiline_input()?;

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
        match agent.generate_response(&user_input).await {
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
