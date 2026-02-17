Auto-Organize

A command-line tool to automatically organize files in a directory by sorting them into categorized folders based on file extensions.
Features

    Automatically organizes files into categorized folders (Images, Documents, Videos, Audio, Archives, Code, Others)

    Dry-run mode to preview changes without actually moving files

    Simple and intuitive command-line interface

    Safe operation - creates directories only when needed

Installation
bash

# Clone the repository
git clone https://github.com/gwangwwan/auto-organize.git
cd auto-organize

# Build and install
cargo install --path .
Usage
bash

# Organize files in the current directory
auto-organize

# Organize files in a specific directory
auto-organize /path/to/directory

# Preview changes without actually moving files
auto-organize -d
auto-organize --dry-run /path/to/directory

# Display help
auto-organize -h
auto-organize --help

# Display version information
auto-organize -V
auto-organize --version
