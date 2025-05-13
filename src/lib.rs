// ShellAI Library

// Export the modules
pub mod agents;
pub mod utils;

// Re-export commonly used items for convenience
pub use agents::openai::OpenAIAgent;
pub use utils::directory;
