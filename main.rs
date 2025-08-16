// src/main.rs - Secure File Backup Utility in Rust
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{DateTime, Local};
use regex::Regex;

/// Result type for our operations
type BackupResult<T> = Result<T, BackupError>;

/// Custom error type for better error handling
#[derive(Debug)]
enum BackupError {
    InvalidFilename(String),
    FileNotFound(String),
    IoError(io::Error),
    PathTraversal(String),
    PermissionDenied(String),
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BackupError::InvalidFilename(msg) => write!(f, "Invalid filename: {}", msg),
            BackupError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            BackupError::IoError(err) => write!(f, "IO Error: {}", err),
            BackupError::PathTraversal(msg) => write!(f, "Path traversal attempt: {}", msg),
            BackupError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl From<io::Error> for BackupError {
    fn from(err: io::Error) -> Self {
        BackupError::IoError(err)
    }
}

/// Secure logging function with timestamps
fn log_action(action: &str) -> BackupResult<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logfile.txt")?;
    
    let timestamp: DateTime<Local> = Local::now();
    let log_entry = format!("[{}] {}\n", timestamp.format("%Y-%m-%d %H:%M:%S"), action);
    
    file.write_all(log_entry.as_bytes())?;
    Ok(())
}

/// Validates filename to prevent security issues
fn validate_filename(filename: &str) -> BackupResult<()> {
    // Check for empty filename
    if filename.is_empty() {
        return Err(BackupError::InvalidFilename("Filename cannot be empty".to_string()));
    }
    
    // Check for path traversal attempts
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        log_action(&format!("Security: Path traversal attempt blocked - {}", filename))?;
        return Err(BackupError::PathTraversal(filename.to_string()));
    }
    
    // Check for valid characters (alphanumeric, dots, dashes, underscores)
    let valid_pattern = Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap();
    if !valid_pattern.is_match(filename) {
        return Err(BackupError::InvalidFilename(
            "Filename contains invalid characters".to_string()
        ));
    }
    
    // Check filename length
    if filename.len() > 255 {
        return Err(BackupError::InvalidFilename(
            "Filename too long (max 255 characters)".to_string()
        ));
    }
    
    Ok(())
}

/// Creates a secure backup of the specified file
fn backup_file(filename: &str) -> BackupResult<()> {
    // Validate filename first
    validate_filename(filename)?;
    
    // Check if source file exists
    let source_path = Path::new(filename);
    if !source_path.exists() {
        log_action(&format!("Backup failed: File not found - {}", filename))?;
        return Err(BackupError::FileNotFound(filename.to_string()));
    }
    
    // Check if we can read the file
    if !source_path.is_file() {
        log_action(&format!("Backup failed: Not a regular file - {}", filename))?;
        return Err(BackupError::InvalidFilename(
            "Target is not a regular file".to_string()
        ));
    }
    
    // Create backup filename
    let backup_name = format!("{}.bak", filename);
    
    // Perform the backup using secure binary copy
    let source_file = File::open(filename)?;
    let mut reader = BufReader::new(source_file);
    
    let dest_file = File::create(&backup_name)?;
    let mut writer = BufWriter::new(dest_file);
    
    // Copy file contents
    let bytes_copied = io::copy(&mut reader, &mut writer)?;
    
    // Ensure all data is written
    writer.flush()?;
    
    println!("Your backup created: {}", backup_name);
    log_action(&format!(
        "Backup successful: {} -> {} ({} bytes)", 
        filename, backup_name, bytes_copied
    ))?;
    
    Ok(())
}

/// Restores a file from its backup
fn restore_file(filename: &str) -> BackupResult<()> {
    // Validate filename
    validate_filename(filename)?;
    
    // Create backup filename
    let backup_name = format!("{}.bak", filename);
    let backup_path = Path::new(&backup_name);
    
    // Check if backup exists
    if !backup_path.exists() {
        log_action(&format!("Restore failed: Backup not found - {}", backup_name))?;
        return Err(BackupError::FileNotFound(backup_name));
    }
    
    // Perform the restoration
    let source_file = File::open(&backup_name)?;
    let mut reader = BufReader::new(source_file);
    
    let dest_file = File::create(filename)?;
    let mut writer = BufWriter::new(dest_file);
    
    // Copy file contents
    let bytes_copied = io::copy(&mut reader, &mut writer)?;
    
    // Ensure all data is written
    writer.flush()?;
    
    println!("File restored from: {}", backup_name);
    log_action(&format!(
        "Restore successful: {} -> {} ({} bytes)", 
        backup_name, filename, bytes_copied
    ))?;
    
    Ok(())
}

/// Securely deletes a file after confirmation
fn delete_file(filename: &str) -> BackupResult<()> {
    // Validate filename
    validate_filename(filename)?;
    
    // Check if file exists
    let file_path = Path::new(filename);
    if !file_path.exists() {
        log_action(&format!("Delete failed: File not found - {}", filename))?;
        return Err(BackupError::FileNotFound(filename.to_string()));
    }
    
    // Get user confirmation
    print!("Are you sure you want to delete {}? (yes/no): ", filename);
    io::stdout().flush()?;
    
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    let confirm = confirm.trim().to_lowercase();
    
    if confirm == "yes" {
        // Attempt to delete the file
        match fs::remove_file(filename) {
            Ok(_) => {
                println!("File deleted.");
                log_action(&format!("Delete successful: {}", filename))?;
                Ok(())
            }
            Err(e) => {
                log_action(&format!("Delete failed: {} - {}", filename, e))?;
                Err(BackupError::from(e))
            }
        }
    } else {
        println!("Delete cancelled.");
        log_action(&format!("Delete cancelled by user: {}", filename))?;
        Ok(())
    }
}

/// Main entry point
fn main() {
    println!("SafeBackup - Secure File Backup Utility (Rust Edition)");
    println!("======================================================");
    
    // Log session start
    if let Err(e) = log_action("SafeBackup session started") {
        eprintln!("Warning: Could not write to log file: {}", e);
    }
    
    // Get filename from user
    print!("Please enter your file name: ");
    io::stdout().flush().expect("Failed to flush stdout");
    
    let mut filename = String::new();
    io::stdin().read_line(&mut filename)
        .expect("Failed to read filename");
    let filename = filename.trim();
    
    // Validate filename early
    if let Err(e) = validate_filename(filename) {
        eprintln!("Error: {}", e);
        let _ = log_action(&format!("Session terminated: {}", e));
        std::process::exit(1);
    }
    
    // Get command from user
    print!("Please enter your command (backup, restore, delete): ");
    io::stdout().flush().expect("Failed to flush stdout");
    
    let mut command = String::new();
    io::stdin().read_line(&mut command)
        .expect("Failed to read command");
    let command = command.trim().to_lowercase();
    
    // Execute command
    let result = match command.as_str() {
        "backup" => backup_file(filename),
        "restore" => restore_file(filename),
        "delete" => delete_file(filename),
        _ => {
            eprintln!("Unknown command: {}", command);
            let _ = log_action(&format!("Unknown command attempted: {}", command));
            std::process::exit(1);
        }
    };
    
    // Handle result
    match result {
        Ok(_) => {
            let _ = log_action("Operation completed successfully");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            let _ = log_action(&format!("Operation failed: {}", e));
            std::process::exit(1);
        }
    }
    
    // Log session end
    let _ = log_action("SafeBackup session ended");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    
    #[test]
    fn test_validate_filename_valid() {
        assert!(validate_filename("test.txt").is_ok());
        assert!(validate_filename("file-name_123.dat").is_ok());
    }
    
    #[test]
    fn test_validate_filename_path_traversal() {
        assert!(validate_filename("../etc/passwd").is_err());
        assert!(validate_filename("../../file.txt").is_err());
        assert!(validate_filename("/etc/passwd").is_err());
        assert!(validate_filename("C:\\\\Windows\\\\System32\\\\file.txt").is_err());
    }
    
    #[test]
    fn test_validate_filename_invalid_chars() {
        assert!(validate_filename("file;rm -rf").is_err());
        assert!(validate_filename("file&command").is_err());
        assert!(validate_filename("file|pipe").is_err());
    }
}
