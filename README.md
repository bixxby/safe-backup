# SafeBackup - Secure File Backup Utility (Rust Edition)

## Part B - Rust Implementation

This is the secure Rust implementation of SafeBackup that addresses all security vulnerabilities found in the C++ version.

## Quick Start

```bash
# Build the project
cargo build --release

# Run the application
cargo run

# Run tests
cargo test

# Run automated test script
./test_safe_backup.sh
```

## Security Features

- Memory safety guaranteed by Rust
- Path traversal protection
- Input validation with regex
- Secure error handling
- Timestamped logging
- No buffer overflows possible
- No format string vulnerabilities
- No use-after-free errors

## Usage

```
$ ./target/release/safe_backup
Please enter your file name: document.txt
Please enter your command (backup, restore, delete): backup
Your backup created: document.txt.bak
```

## Testing

Run all tests with:
```bash
cargo test
```

## Author

[Your Name]
