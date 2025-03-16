#!/usr/bin/env python3
"""
pybacon_vscode_seq.py

Intent:
    This script is designed to improve workflow when working with Rust projects.
    It first opens the current workspace (repository folder) in Visual Studio Code (VSCode) so that
    Rust Analyzer only needs to parse the project once. Then, it sequentially opens files extracted
    from a bacon file (which contains warning messages with file paths, line numbers, and column numbers)
    in the already open VSCode workspace. The script pauses after opening each file until you close that
    file's editor, allowing you to inspect the file before moving on.

Functionality:
    1. Verifies that the VSCode command-line tool ('code') is available in the system PATH.
    2. Opens the current folder in VSCode (the workspace) without blocking.
    3. Waits a few seconds to allow the workspace to load fully.
    4. Reads the bacon file line by line and uses a regular expression to extract the file path, line, and column.
    5. For each extracted file location, uses VSCode’s --goto flag to open the file at the specified line and column,
       and --wait so that the script pauses until that file’s editor tab is closed.
       
Usage:
    python pybacon_vscode_seq.py <path_to_bacon_file>
    
Prerequisites:
    - Visual Studio Code must be installed.
    - The 'code' command must be available in your system PATH.
    - The bacon file should contain warning messages in the following format:
         warning <file_path>:<line>:<column> ...

Example:
    python pybacon_vscode_seq.py .bacon-locations

Author:
    Your Name
Date:
    YYYY-MM-DD
"""

import sys
import re
import subprocess
import shutil
import time

def main():
    # Ensure that a bacon file path is provided.
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <path_to_bacon_file>")
        sys.exit(1)

    bacon_file = sys.argv[1]

    # Verify that the 'code' command is available in PATH.
    if shutil.which("code") is None:
        print("Error: 'code' command not found in PATH. Please ensure VSCode is installed and the command is added to your PATH.")
        sys.exit(1)

    # Open the current directory as a VSCode workspace.
    print("Opening current workspace in VSCode...")
    subprocess.Popen("code .", shell=True)
    
    # Allow time for VSCode to load the workspace fully.
    print("Waiting for workspace to load...")
    time.sleep(5)  # Adjust the delay if needed for your system.

    # Regex pattern to capture the file path, line, and column from each warning line.
    pattern = re.compile(r"^warning\s+(.*):(\d+):(\d+)\b")

    # Process the bacon file line by line.
    with open(bacon_file, "r", encoding="utf-8") as f:
        for line in f:
            match = pattern.match(line)
            if match:
                file_path = match.group(1)
                line_number = match.group(2)
                col_number = match.group(3)
                target = f"{file_path}:{line_number}:{col_number}"
                print(f"Opening {target} in VSCode (waiting for closure)...")
                # Open the file within the existing VSCode workspace.
                # --goto jumps to the specified line:column.
                # --wait causes the command to block until the file is closed.
                subprocess.run(f"code --goto --wait {target}", shell=True)

if __name__ == '__main__':
    main()

