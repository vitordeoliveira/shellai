// Directory utility functions for ShellAI

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// Scans a directory and builds a tree structure representation
///
/// # Arguments
///
/// * `path` - The path to scan
/// * `max_depth` - Maximum depth to scan (0 means only the top level)
/// * `current_depth` - Current depth in the recursion (should be 0 for initial calls)
///
/// # Returns
///
/// A string representation of the directory tree
pub fn scan_directory(path: &Path, max_depth: usize, current_depth: usize) -> Result<String, Box<dyn Error>> {
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

/// Gets the current working directory
///
/// # Returns
///
/// The current working directory as a PathBuf
pub fn get_current_directory() -> Result<PathBuf, Box<dyn Error>> {
    let current_dir = env::current_dir()?;
    Ok(current_dir)
}

/// Builds a system prompt with directory information
///
/// # Arguments
///
/// * `base_prompt` - The base system prompt to enhance with directory information
///
/// # Returns
///
/// An enhanced system prompt with directory information
pub fn build_directory_aware_prompt(base_prompt: &str) -> Result<String, Box<dyn Error>> {
    let current_dir = get_current_directory()?;
    let dir_name = current_dir.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    let dir_path = current_dir.to_string_lossy().to_string();
    
    // Scan the directory structure (limit depth to 2 to avoid overwhelming output)
    let dir_tree = scan_directory(&current_dir, 2, 0)?;
    
    let prompt = format!(r#"Current working directory: {}
Directory name: {}

Directory structure:
{}

{}

Additional guidelines:
- Be aware of the current directory structure shown above when suggesting commands.
- When referencing files or directories, use the correct paths based on the current directory."#, 
        dir_path, dir_name, dir_tree, base_prompt);

    Ok(prompt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

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
    fn test_build_directory_aware_prompt() {
        // This is a basic test to ensure the function runs without errors
        let base_prompt = "This is a test prompt.";
        let result = build_directory_aware_prompt(base_prompt);
        assert!(result.is_ok());
        
        let prompt = result.unwrap();
        assert!(prompt.contains("Current working directory:"));
        assert!(prompt.contains("Directory name:"));
        assert!(prompt.contains("Directory structure:"));
        assert!(prompt.contains(base_prompt));
    }
}
