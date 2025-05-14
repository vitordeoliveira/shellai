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

// Define available AI models/agents
#[derive(Debug, Clone)]
struct AIModel {
    name: String,
    model_id: String,
    description: String,
}

// List of available AI models
fn get_available_models() -> Vec<AIModel> {
    vec![
        AIModel {
            name: "GPT-4".to_string(),
            model_id: "gpt-4".to_string(),
            description: "Advanced model with strong reasoning capabilities".to_string(),
        },
        AIModel {
            name: "GPT-3.5 Turbo".to_string(),
            model_id: "gpt-3.5-turbo".to_string(),
            description: "Fast and efficient for most tasks".to_string(),
        },
        AIModel {
            name: "GPT-4o".to_string(),
            model_id: "gpt-4o".to_string(),
            description: "Latest model with improved capabilities".to_string(),
        },
    ]
}

/// Display available AI models and let the user select one
fn select_ai_model() -> Result<Option<AIModel>, Box<dyn std::error::Error>> {
    let models = get_available_models();

    println!("\n{}", "Available AI Models:".bright_yellow());
    println!("{}", "─".repeat(60).bright_black());

    for (i, model) in models.iter().enumerate() {
        println!(
            "{}: {} - {}",
            (i + 1).to_string().bright_cyan(),
            model.name.bright_green(),
            model.description.bright_white()
        );
    }

    println!("{}", "─".repeat(60).bright_black());
    print!(
        "{}: ",
        "Enter model number (or any other key to cancel)".bright_yellow()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let selection = input.trim().parse::<usize>().ok();

    match selection {
        Some(n) if n > 0 && n <= models.len() => Ok(Some(models[n - 1].clone())),
        _ => {
            println!("{}", "Model selection cancelled.".bright_yellow());
            Ok(None)
        }
    }
}

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
                        println!("\n{}", "Goodbye!".bright_blue());
                        std::process::exit(0); // Immediately exit the program
                    }

                    // Handle Ctrl+A to show available models (A for Agents)
                    if c == 'a' && modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        return Ok("ctrl+a".to_string());
                    }

                    // Handle Ctrl+h to show expanded menu (h for help)
                    if c == 'h' && modifiers.contains(KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        return Ok("ctrl+h".to_string());
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

    // Default model
    let mut current_model = "gpt-4".to_string();

    // Create an OpenAI agent
    let mut agent = match OpenAIAgent::new(current_model.clone()) {
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
        // Print simplified inline menu
        println!("\n{}", "─".repeat(60).bright_black());
        println!(
            "{} {} {} {} {} {} {} {}",
            "Model:".bright_yellow(),
            current_model.bright_green(),
            "•".bright_white(),
            "Commands:".bright_yellow(),
            "<c-s> to send".bright_cyan(),
            "•".bright_white(),
            "<c-h> for help".bright_cyan(),
            "<c-a> for models".bright_cyan()
        );
        println!("{}", "─".repeat(60).bright_black());

        // Print prompt
        print!("{}: ", "You".bright_green());
        io::stdout().flush()?;

        // Read multiline user input
        let user_input = read_multiline_input()?;

        // We're no longer checking for "exit" or "quit" text commands
        // as we prefer to use Ctrl+C for exiting

        // Check for expanded menu command
        if user_input == "ctrl+h" {
            println!("\n{}", "ShellAI Expanded Help:".bright_yellow());
            println!("{}", "─".repeat(60).bright_black());
            println!("{} - Add a new line", "Enter".bright_cyan());
            println!("{} - Submit your question", "Ctrl+S".bright_cyan());
            println!("{} - Exit the application", "Ctrl+C".bright_cyan());
            println!("{} - Cancel current input", "Esc".bright_cyan());
            println!("{} - Navigate and edit text", "Backspace".bright_cyan());
            println!("{} - Show this expanded help menu", "Ctrl+H".bright_cyan());
            println!("{} - Select a different AI model", "Ctrl+A".bright_cyan());
            println!("{}", "─".repeat(60).bright_black());
            continue;
        }

        // Check for model selection command
        if user_input == "ctrl+a" {
            match select_ai_model()? {
                Some(model) => {
                    println!(
                        "{} {}",
                        "Switching to model:".bright_yellow(),
                        model.name.bright_green()
                    );
                    current_model = model.model_id.clone();

                    // Create a new agent with the selected model
                    agent = match OpenAIAgent::new(current_model.clone()) {
                        Ok(new_agent) => new_agent,
                        Err(e) => {
                            eprintln!("Error initializing OpenAI agent with new model: {}", e);
                            continue;
                        }
                    };
                }
                None => {
                    println!(
                        "{} {}",
                        "Continuing with current model:".bright_yellow(),
                        current_model.bright_green()
                    );
                }
            }
            continue;
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
}
