# ShellAI

ShellAI is a terminal-based AI assistant that brings the power of large language models directly to your command line. It allows you to interact with AI models like GPT-4 and GPT-3.5 Turbo right from your terminal, making it easy to get help with coding, command suggestions, and more without leaving your workflow.

![ShellAI Demo](https://github.com/yourusername/shellai/raw/main/assets/demo.gif)

## Features

- 🤖 Interact with powerful AI models directly in your terminal
- 🔄 Switch between different AI models (GPT-4, GPT-3.5 Turbo, GPT-4o)
- 💻 Execute bash code suggestions with a simple confirmation
- 📝 Multi-line input support for complex queries
- 🎨 Colorful, user-friendly interface
- ⌨️ Convenient keyboard shortcuts

## Installation

### Prerequisites

- Rust and Cargo (install via [rustup](https://rustup.rs/))
- An OpenAI API key

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/shellai.git
   cd shellai
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. The compiled binary will be available at `target/release/shellai`

4. (Optional) Move the binary to a directory in your PATH:
   ```bash
   sudo mv target/release/shellai /usr/local/bin/
   ```

### Setting up your OpenAI API Key

ShellAI requires an OpenAI API key to function. You can set it as an environment variable:

```bash
export OPENAI_API_KEY="your-api-key-here"
```

For permanent setup, add this line to your shell profile file (`~/.bashrc`, `~/.zshrc`, etc.).

## Usage

### Basic Usage

Simply run the `shellai` command in your terminal:

```bash
shellai
```

### Keyboard Shortcuts

- `<c-s>` (Ctrl+S): Submit your question
- `<c-a>` (Ctrl+A): Select a different AI model
- `<c-h>` (Ctrl+H): Show the expanded help menu
- `<c-c>` (Ctrl+C): Exit the application
- `Enter`: Add a new line
- `Esc`: Cancel current input
- `Backspace`: Navigate and edit text

### Executing Code

When ShellAI provides bash code in its response, it will be highlighted and you'll be prompted with an option to execute it directly.

## Binding to Ctrl+A in Your Shell

### For Bash

Add the following to your `~/.bashrc` file:

```bash
# ShellAI binding (Ctrl+A)
shellai_run() {
  # Save current line
  local line=$READLINE_LINE
  local point=$READLINE_POINT
  
  # Clear the line and run shellai
  READLINE_LINE=""
  echo
  shellai
  
  # Restore the line
  READLINE_LINE=$line
  READLINE_POINT=$point
}

# Bind Ctrl+A to shellai
bind -x '"\C-a": shellai_run'
```

### For Zsh

Add the following to your `~/.zshrc` file:

```zsh
# ShellAI binding (Ctrl+A)
shellai_run() {
  # Save current buffer
  local buffer=$BUFFER
  local cursor=$CURSOR
  
  # Clear the line and run shellai
  BUFFER=""
  zle -R
  echo
  shellai
  
  # Restore the buffer
  BUFFER=$buffer
  CURSOR=$cursor
  zle redisplay
}

# Create a widget and bind it to Ctrl+A
zle -N shellai_run
bindkey '^A' shellai_run
```

After adding these configurations, restart your shell or source the configuration file:

```bash
# For Bash
source ~/.bashrc

# For Zsh
source ~/.zshrc
```

## Troubleshooting

### API Key Issues

If you see an error about the API key, make sure your `OPENAI_API_KEY` environment variable is set correctly:

```bash
echo $OPENAI_API_KEY
```

### Terminal Compatibility

ShellAI uses the `crossterm` library for terminal handling, which supports most modern terminals. If you experience display issues, try using a different terminal emulator.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with Rust and the OpenAI API
- Uses the `crossterm` library for terminal handling
- Inspired by the need for AI assistance directly in the terminal
