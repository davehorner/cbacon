#!/usr/bin/env python3
"""
pybacon_vscode.py

Intent:
    This script reads a "bacon file" containing warning messages that include file paths with line and column numbers.
    It extracts these details from each warning and uses them to open the specified files in Visual Studio Code (VSCode).

Functionality:
    - Parses each line of the bacon file to extract:
        • File path (supports Windows-style paths)
        • Line number
        • Column number
    - Constructs a target string in the format "file_path:line:column".
    - Uses the VSCode command-line interface to open each file:
        • Opens the file in a new VSCode window (-n).
        • Positions the cursor at the specified line and column (--goto).
        • Waits for the VSCode window to be closed (--wait) before moving on to the next file.
        
Usage:
    python pybacon_vscode.py <path_to_bacon_file>
    
Prerequisites:
    - Visual Studio Code must be installed.
    - The 'code' command must be available in your system PATH. You can add it from within VSCode using the
      "Shell Command: Install 'code' command in PATH" command.
      
Example:
    python pybacon_vscode.py .bacon-locations

Author:
    Your Name
Date:
    YYYY-MM-DD
"""

import sys
import re
import subprocess
import shutil

def main():
    # Ensure the script is called with the required bacon file argument.
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <path_to_bacon_file>")
        sys.exit(1)

    # Check if the 'code' command is available in the system PATH.
    if shutil.which("code") is None:
        print("Error: 'code' command not found in PATH. Please ensure VSCode is installed and the command is added to your PATH.")
        sys.exit(1)

    bacon_file = sys.argv[1]
    
    # Regex to capture:
    #   group(1): file path (supports Windows paths like C:\...)
    #   group(2): line number
    #   group(3): column number
    pattern = re.compile(r"^warning\s+(.*):(\d+):(\d+)\b")

    # Open and parse the bacon file.
    with open(bacon_file, "r", encoding="utf-8") as f:
        for line in f:
            match = pattern.match(line)
            if match:
                file_path = match.group(1)
                line_number = match.group(2)
                col_number = match.group(3)
                target = f"{file_path}:{line_number}:{col_number}"
                print(f"Opening {target} in VSCode (new window, waiting for closure)...")
                # Launch VSCode:
                #   - '-n': Open in a new window.
                #   - '--goto': Open the file at the specified line:column.
                #   - '--wait': Pause execution until the opened window is closed.
                subprocess.run(f"code -n --goto --wait {target}", shell=True)

if __name__ == '__main__':
    main()

