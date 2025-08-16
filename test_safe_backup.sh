#!/bin/bash

echo "╔════════════════════════════════════════════╗"
echo "║     SafeBackup Automated Test Script       ║"
echo "╚════════════════════════════════════════════╝"

# Build the project
cargo build --release

# Create test files
echo "Test content" > test1.txt

# Test backup
echo -e "test1.txt\nbackup" | ./target/release/safe_backup

# Check if backup was created
if [ -f "test1.txt.bak" ]; then
    echo "✓ Backup test passed"
else
    echo "✗ Backup test failed"
fi

# Cleanup
rm -f test*.txt test*.txt.bak

echo "Basic tests completed!"
