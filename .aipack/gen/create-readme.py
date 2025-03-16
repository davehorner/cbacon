#!/usr/bin/env python3
def main():
    placeholder = "<<<TICKS>>>"
    # Define the README content using the placeholder token in place of triple backticks.
    content = f"""# cbacon

**cbacon** is a Rust-based tool that reads a bacon file containing warning messages and opens the first warning location in Visual Studio Code (VSCode). It emulates a Python script by automatically opening your project workspace in VSCode, parsing warning messages from a file, and navigating directly to the relevant file location using VSCode's --goto and --wait flags.

## Features

- **Workspace Integration:** Opens the current directory as a VSCode workspace so that Rust Analyzer and other extensions process the project.
- **Warning File Parsing:** Reads a bacon file (default: .bacon-locations) containing warning messages formatted as:
  
  warning <file_path>:<line>:<column> <message>
  
- **File Navigation:** Opens the first warning location in VSCode and waits until you close the file before re-checking for updates.
- **Continuous Monitoring:** Automatically re-reads the bacon file for any updates after the warning file is closed.
- **File Watcher Support:** Optionally uses a file system watcher (via the notify crate) to monitor for modifications in the bacon file.

## Installation

Ensure you have [Rust](https://www.rust-lang.org/) installed. Then, clone the repository and build the project:

{placeholder}bash
git clone <repository-url>
cd cbacon
cargo build --release
{placeholder}

Replace <repository-url> with the actual URL of the repository.

## Usage

Run **cbacon** with an optional path to the bacon file. If no path is specified, it defaults to .bacon-locations in the current directory:

{placeholder}bash
cargo run -- <path_to_bacon_file>
{placeholder}

For example:

{placeholder}bash
cargo run -- .bacon-locations
{placeholder}

> Note: Make sure the "code" command (VSCode) is available in your system's PATH. Verify with:

{placeholder}bash
code --version
{placeholder}

## Testing

The project includes a suite of tests to ensure proper functionality. Run the tests with:

{placeholder}bash
cargo test
{placeholder}

The tests cover:
- Parsing of warning messages from the bacon file.
- Correct handling of file paths and line numbers.
- Integration with the VSCode command.
- File watcher behavior when using the uses_notify feature.

## License

**cbacon** is licensed under the MIT License. See the LICENSE file for more details.

## Author

David Horner

## Last Updated

March 15, 2025
"""
    # Replace the placeholder token with the actual triple backticks.
    final_content = content.replace(placeholder, "```")
    
    # Write the final content to README.md
    with open("README.md", "w", encoding="utf-8") as f:
        f.write(final_content)
    print("README.md has been created successfully.")

if __name__ == '__main__':
    main()

