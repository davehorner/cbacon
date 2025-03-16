# cbacon

**cbacon** is a Rust-based tool that reads a bacon file containing line references and opens the first reference location in Visual Studio Code (VSCode). It emulates a Python script by automatically opening your project workspace in VSCode, parsing warning/error messages from a file, and navigating directly to the relevant file location using VSCode's --goto and --wait flags.

[bacon](https://github.com/Canop/bacon) is a background code checker. It's designed for minimal interaction so that you can just let it run, alongside your editor, and be notified of warnings.

I've been using a simple python script to provide an experience much like nvim-bacon but for vscode.  Run bacon, run cbacon.  I decided to make it into a rust program too.

Combined with bacon, cbacon navigates between errors and warnings without leaving your editor, just hitting a key. (close the current editor/workspace)

I would like to see something like this integrated into bacon itself, but in the meantime, here is how I use bacon sometimes.

## Features

- **Workspace Integration:** Opens the current directory as a VSCode workspace so that Rust Analyzer and other extensions process the project.
- **Warning File Parsing:** Reads a bacon file (default: .bacon-locations) containing warning messages formatted as:
  
  warning <file_path>:<line>:<column> <message>
  
- **File Navigation:** Opens the first warning location in VSCode and waits until you close the file before re-checking for updates.
- **Continuous Monitoring:** Automatically re-reads the bacon file for any updates after the warning file is closed.
- **File Watcher Support:** Optionally uses a file system watcher (via the notify crate) to monitor for modifications in the bacon file.

## Installation

Ensure you have [Rust](https://www.rust-lang.org/) installed. Then, clone the repository and build the project:

```bash
git clone <repository-url>
cd cbacon
cargo build --release
```

Replace <repository-url> with the actual URL of the repository.

## Usage

Run **cbacon** with an optional path to the bacon file. If no path is specified, it defaults to .bacon-locations in the current directory:

```bash
cargo run -- <path_to_bacon_file>
```

For example:

```bash
cargo run -- .bacon-locations
```

> Note: Make sure the "code" command (VSCode) is available in your system's PATH. Verify with:

```bash
code --version
```

## Testing

The project includes a suite of tests to ensure proper functionality. Run the tests with:

```bash
cargo test
```

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
