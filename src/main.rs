//! A tool to read a bacon file of warning messages and open the first warning location in VSCode.
//!
//! # Overview
//!
//! This program emulates the functionality of a Python script that:
//!
//! 1. Opens the current directory as a VSCode workspace so that Rust Analyzer processes the project once.
//! 2. Reads a bacon file (by default, `.bacon-locations`) containing warning messages in the format:
//!    `warning <file_path>:<line>:<column> ...`.
//! 3. Opens the first warning location in VSCode using the `--goto` and `--wait` flags (blocking until the file is closed).
//! 4. Once the file is closed, re-reads the bacon file for any updates.
//!
//! Run the program with an optional bacon file path:
//!
//! ```bash
//! cargo run -- <path_to_bacon_file>
//! ```
//! If no file is specified, it defaults to `.bacon-locations` in the current directory.

use regex::Regex;
use std::env;
use std::fs;
use std::process::exit;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Global map tracking the line content of each warning target ("file:line:column").
/// This is used to determine if a warning's target line has changed since it was first processed.
static LINES_OF_CONCERN: Lazy<Mutex<std::collections::HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

/// Opens the current directory as a VSCode workspace.
///
/// This function spawns the `code .` command and then waits for a few seconds to allow
/// the workspace to load.
///
/// # Errors
///
/// Returns an error if the command cannot be spawned.
async fn open_workspace() -> Result<(), Box<dyn std::error::Error>> {
    println!("Opening current workspace in VSCode...");
    open_vscode(".").await;
    Ok(())
}

/// Reads the bacon file and extracts warning entries.
///
/// Each valid warning line should have the format:
/// `warning <file_path>:<line>:<column> ...`
///
/// Returns a vector of strings where each string is formatted as "file_path:line:column".
///
/// # Arguments
///
/// * `bacon_file` - A string slice that holds the path to the bacon file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or the regex pattern fails.
fn read_bacon_file(bacon_file: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Read the entire contents of the file.
    let content = fs::read_to_string(bacon_file)?;
    // This regex will match lines beginning with any non-whitespace token (the type),
    // then one or more spaces, then a file path (non-greedy), a colon, a line number,
    // a colon, and a column number. We ignore the type in the returned target.
    let re = Regex::new(r"^(?:\S+\s+)?(.+?):(\d+):(\d+)\b")?;
    let mut targets = Vec::new();

    for line in content.lines() {
        if let Some(caps) = re.captures(line) {
            // We ignore any leading type and only use file path, line, and column.
            let file_path = &caps[1];
            let line_num = &caps[2];
            let col = &caps[3];
            let target = format!("{}:{}:{}", file_path, line_num, col);
            targets.push(target);
        }
    }
    Ok(targets)
}

/// Opens a file location in VSCode using the `--goto` and `--wait` flags.
/// This function leverages the asynchronous `open_vscode` function to open
/// the given target location, and waits until the file is closed.
///
/// # Arguments
///
/// * `target` - A string slice in the format "file_path:line:column".
///
/// # Errors
///
/// Returns an error if the process fails to spawn (note that `open_vscode` itself
/// prints errors rather than returning them).
pub async fn open_line_ref(target: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Opening {} in VSCode (waiting for closure)...", target);
    // Build the argument string with the required flags and the target.
    let args = format!("--goto --wait {}", target);
    // Call the asynchronous open_vscode function with the combined argument string.
    open_vscode(args).await;
    Ok(())
}

/// The main function that ties everything together.
///
/// It checks for a provided bacon file path (defaults to `.bacon-locations`),
/// ensures that the `code` command is available in the PATH, opens the VSCode workspace,
/// and then continuously monitors the bacon file for warnings.
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let bacon_file = if args.len() < 2 {
        println!("No bacon file specified. Defaulting to .bacon-locations in the current directory.");
        ".bacon-locations".to_string()
    } else {
        args[1].clone()
    };

    // Verify that the 'code' command is available.
    if which::which("code").is_err() {
        eprintln!("Error: 'code' command not found in PATH. Please ensure VSCode is installed and the command is added to your PATH.");
        exit(1);
    }

    if let Err(e) = open_workspace().await {
        eprintln!("Failed to open workspace: {}", e);
        exit(1);
    }

    notify_watch::watch_bacon_file(&bacon_file).ok();
       crate::notify_watch::monitor_linerefs(&bacon_file).await;
    // println!("Monitoring bacon file for warnings. Press Ctrl+C to exit.");
    // loop {
    //     match read_bacon_file(&bacon_file) {
    //         Ok(targets) => {
    //             if !targets.is_empty() {
    //                 let target = &targets[0];
    //                 if let Err(e) = open_line_ref(target).await {
    //                     eprintln!("Error opening warning: {}", e);
    //                 } else {
    //                     println!("Editor closed. Re-reading bacon file for any updates...");
    //                 }
    //             } else {
    //                 println!("No warnings found. Waiting for updates...");
    //                 sleep(Duration::from_secs(2));
    //             }
    //         }
    //         Err(e) => {
    //             eprintln!("Error reading bacon file: {}", e);
    //             sleep(Duration::from_secs(2));
    //         }
    //     }
    // }
}

/// Attempt to open a path via VSCode with additional arguments.
///
/// # Parameters
///
/// * `args`: A string slice containing all arguments to pass to VSCode.  
///   For example: `"--goto /path/to/file:12:34 --wait"`
///
/// On Windows, the command is executed via `cmd /C code <args>`.
/// On other platforms, it calls `code <args>`.
pub async fn open_vscode(args: impl AsRef<str>) {
    use std::process::Command;

    let args_str = args.as_ref();
    let tokens: Vec<&str> = args_str.split_whitespace().collect();

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", "code"])
            .args(&tokens)
            .output()
    } else {
        Command::new("code")
            .args(&tokens)
            .output()
    };

    match output {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            let msg = format!(
                "Error opening VSCode:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            println!("{}", msg);
        }
        Err(e) => {
            let msg = format!("Failed to execute VSCode command: {}", e);
            println!("{}", msg);
        }
    }
}


// /// Attempt to open a path via VSCode with additional arguments.
// ///
// /// # Parameters
// ///
// /// * `args`: A string slice containing all arguments to pass to VSCode.  
// ///   For example: `"--goto /path/to/file:12:34 --wait"`
// ///
// /// On Windows, the command is executed via `cmd /C code <args>`.
// /// On other platforms, it calls `code <args>`.
// pub async fn open_vscode(args: impl AsRef<str>) {
//     use std::process::Command;
//     use std::str;

//     // Split the provided arguments string into individual tokens.
//     let args_str = args.as_ref();
//     let tokens: Vec<&str> = args_str.split_whitespace().collect();

//     let output = if cfg!(target_os = "windows") {
//         // On Windows, run via "cmd /C code ..." using the tokens.
//         Command::new("cmd")
//             .args(&["/C", "code"])
//             .args(&tokens)
//             .output()
//     } else {
//         // On Linux/Mac, run "code ..." directly.
//         Command::new("code")
//             .args(&tokens)
//             .output()
//     };

//     match output {
//         Ok(output) if output.status.success() => {}
//         Ok(output) => {
//             let msg = format!(
//                 "Error opening VSCode:\nstdout: {}\nstderr: {}",
//                 String::from_utf8_lossy(&output.stdout),
//                 String::from_utf8_lossy(&output.stderr)
//             );
//             println!("{}", msg);
//         }
//         Err(e) => {
//             let msg = format!("Failed to execute VSCode command: {}", e);
//             println!("{}", msg);
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that a valid bacon file content produces the expected target string.
    #[test]
    fn test_read_bacon_file_valid() {
        let test_content = "\
warning /path/to/file.rs:23:17 Some warning message
warning /another/path/file.rs:45:2 Another warning message
";
        // Write test content to a temporary file.
        let tmp_file = "test_bacon.txt";
        fs::write(tmp_file, test_content).unwrap();

        let targets = read_bacon_file(tmp_file).unwrap();
        // Clean up the temporary file.
        fs::remove_file(tmp_file).unwrap();

        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0], "/path/to/file.rs:23:17");
        assert_eq!(targets[1], "/another/path/file.rs:45:2");
    }

    /// Test that an empty bacon file returns an empty vector.
    #[test]
    fn test_read_bacon_file_empty() {
        let test_content = "";
        let tmp_file = "test_empty_bacon.txt";
        fs::write(tmp_file, test_content).unwrap();

        let targets = read_bacon_file(tmp_file).unwrap();
        fs::remove_file(tmp_file).unwrap();

        assert!(targets.is_empty());
    }

    /// Test that lines not matching the warning format are ignored.
    #[test]
    fn test_read_bacon_file_invalid_format() {
        let test_content = "\
This is not a valid warning line
warning no_colon_or_numbers
warning /file/path:line:col Extra text that doesn't match
";
        let tmp_file = "test_invalid_bacon.txt";
        fs::write(tmp_file, test_content).unwrap();

        let targets = read_bacon_file(tmp_file).unwrap();
        fs::remove_file(tmp_file).unwrap();

        // In this case, none of the lines match the expected pattern.
        assert!(targets.is_empty());
    }

    /// Test running the binary with the `--version` flag to ensure version information is printed
    /// in the correct format.
    ///
    /// This test uses the current executable to simulate the command line call.
    #[test]
    fn test_bacon_version_flag() {
        let output = std::process::Command::new("bacon")
            .arg("--version")
            .output()
            .expect("failed to execute process");
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.is_empty() {
            eprintln!("DEBUG: No output captured. Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        // Check that the output meets the form "bacon X.Y.Z"
        let version_regex = Regex::new(r"^bacon\s+\d+\.\d+\.\d+").unwrap();
        assert!(
            version_regex.is_match(stdout.trim()),
            "Version output did not match expected pattern. Got: '{}'",
            stdout
        );
    }

    #[test]
fn test_code_version_output_with_cmd() {
    use regex::Regex;
    use std::process::Command;
    
    // Depending on the operating system, use the appropriate command to run "code --version".
    // On Windows, we use "cmd /C code --version", while on Linux/Mac we call "code --version" directly.
    let output = if cfg!(target_os = "windows") {
        // On Windows, use the cmd shell to execute the command.
        Command::new("cmd")
            .args(&["/C", "code", "--version"])
            .output()
            .expect("Failed to execute 'cmd /C code --version'. Ensure VSCode's 'code' command is available in PATH.")
    } else {
        // On Linux and macOS, call "code --version" directly.
        Command::new("code")
            .arg("--version")
            .output()
            .expect("Failed to execute 'code --version'. Ensure VSCode's 'code' command is available in PATH.")
    };

    // Convert the command output (bytes) into a string.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Debug prints to help trace the command output.
    println!("DEBUG: Raw stdout: {}", stdout);
    println!("DEBUG: Raw stderr: {}", stderr);

    // Split the output into lines. We expect the first line to be the version number (e.g., "1.97.2").
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        !lines.is_empty(),
        "Expected at least one line of output from 'code --version', but got none."
    );

    // Define a regex to match a semantic version number (e.g., "1.97.2").
    let version_regex = Regex::new(r"^\d+\.\d+\.\d+$").unwrap();

    // Check that the first line (version line) matches the semantic version pattern.
    assert!(
        version_regex.is_match(lines[0]),
        "Version line does not match the expected pattern. Got: '{}'",
        lines[0]
    );

    // Print the detected version for debugging purposes.
    println!("Detected VSCode version: {}", lines[0]);
}


#[test]
fn test_read_bacon_locations_with_sample_rs() {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    // Create a temporary "testgen" folder.
    let testgen_dir = Path::new("testgen");
    fs::create_dir_all(&testgen_dir).expect("Failed to create testgen directory");

    // Create a sample.rs file with some valid Rust code.
    // We'll create 12 lines of code so that our warning line numbers refer to valid lines.
    let sample_rs_content = "\
// sample.rs file - example code
fn main() {
    let a = 1;
    let unused_var = 2;        // Line 4: intended for warning
    let b = a + 1;
    println!(\"{}\", b);
    let c = 3;
    let another_unused = 4;    // Line 8: intended for warning
    let d = 5;
    // some comment
    let e = 6;                // Line 11: intended for warning
}
";
    let sample_rs_path = testgen_dir.join("sample.rs");
    {
        let mut sample_file = File::create(&sample_rs_path).expect("Failed to create sample.rs file");
        sample_file
            .write_all(sample_rs_content.as_bytes())
            .expect("Failed to write to sample.rs file");
    }

    // Get the canonical (absolute) path of the sample.rs file.
    let canonical_sample_path = fs::canonicalize(&sample_rs_path)
        .expect("Failed to canonicalize sample.rs path")
        .to_string_lossy()
        .to_string();

    // Create sample warnings that reference valid lines in the sample.rs file.
    // These warnings follow the format:
    //   warning <file_path>:<line>:<column> <message>
    // We use line 4, 8, and 11 as they correspond to our intended warnings in sample.rs.
    let warnings = vec![
        format!("warning {}:4:9 unused variable: `unused_var`", canonical_sample_path),
        format!("warning {}:8:9 unused variable: `another_unused`", canonical_sample_path),
        format!("warning {}:11:9 unused variable: `e`", canonical_sample_path),
    ];
    let warnings_content = warnings.join("\n");

    // Write the warnings to a .bacon-locations file inside the testgen folder.
    let bacon_file_path = testgen_dir.join("bacon-locations.txt");
    {
        let mut bacon_file = File::create(&bacon_file_path)
            .expect("Failed to create .bacon-locations file");
        bacon_file
            .write_all(warnings_content.as_bytes())
            .expect("Failed to write to .bacon-locations file");
    }

    // Call the read_bacon_file function (assumed to be in scope) to parse the .bacon-locations file.
    let targets = read_bacon_file(bacon_file_path.to_str().unwrap())
        .expect("Failed to read .bacon-locations file");

    // Build the expected vector of targets.
    let expected_targets = vec![
        format!("{}:4:9", canonical_sample_path),
        format!("{}:8:9", canonical_sample_path),
        format!("{}:11:9", canonical_sample_path),
    ];

    // Assert that the parsed targets match the expected targets.
    assert_eq!(
        targets, expected_targets,
        "Parsed targets did not match expected targets.\nParsed: {:?}\nExpected: {:?}",
        targets, expected_targets
    );

    // Cleanup: Remove the created files and directory.
    fs::remove_file(bacon_file_path).expect("Failed to remove .bacon-locations file");
    fs::remove_file(&sample_rs_path).expect("Failed to remove sample.rs file");
    fs::remove_dir(&testgen_dir).expect("Failed to remove testgen directory");
}

#[test]
fn test_nonexistent_bacon_file_output() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    // Run the binary (named "cbacon" in this example) with a bacon file that doesn't exist.
    // This should trigger the error branch in our main loop, printing an error message.
    let mut cmd = Command::cargo_bin("cbacon").expect("Binary 'cbacon' not found");
    cmd.arg("nonexistent_bacon_file.txt");

    // Assert that the command fails and that stderr contains our expected error message.
    // Adjust the expected string if your implementation prints a different message.
    cmd.assert()
       .failure()
       .stderr(predicate::str::contains("Error reading bacon file"));
}

#[test]
fn test_nonexistent_bacon_file_output__() {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use assert_cmd::cargo::cargo_bin;
    use which::which;

    // Determine which error messages are acceptable.
    // If a VSCode command is found, we expect an error reading the bacon file.
    // Otherwise, we expect an error about failing to open the workspace.
    let expected_errors = if which("code").is_ok() || which("code.cmd").is_ok() || which("code.exe").is_ok() {
        vec!["Error reading bacon file", "Failed to open workspace"] // sometimes spawn may fail later than reading the file
    } else {
        vec!["Failed to open workspace"]
    };

    // Get the binary's path using cargo_bin.
    let bin_path = cargo_bin("cbacon");

    // Spawn the process with a bacon file that does not exist.
    let mut child = Command::new(bin_path)
        .arg("nonexistent_bacon_file.txt")
        .spawn()
        .expect("Failed to spawn child process");

    // Allow the process time to produce output.
    thread::sleep(Duration::from_secs(3));

    // Kill the child process, since our main loop never terminates.
    child.kill().expect("Failed to kill child process");

    // Wait for the process to exit and capture its output.
    let output = child.wait_with_output().expect("Failed to wait on child process");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Print the captured stderr for debugging.
    eprintln!("DEBUG: Captured stderr: {}", stderr);

    // Assert that stderr contains at least one of the expected error substrings.
    let found = expected_errors.iter().any(|sub| stderr.contains(sub));
    assert!(
        found,
        "Expected stderr to contain one of {:?}, but got: {}",
        expected_errors,
        stderr
    );
}

}


#[cfg(feature = "uses_notify")]
mod notify_watch {
     use notify::{RecursiveMode, Watcher, EventKind};
    use std::sync::mpsc::channel;
    use std::time::{Duration, Instant};
    use std::path::Path;

    /// Watches the given file for modifications.
    ///
    /// Each time a modify event is received, a message is printed along with the current save count.
    /// If the file is modified twice within 10 seconds, returns Ok(true).
    /// If no second modification is detected within 10 seconds (or after a timeout), returns Ok(false).
    pub fn watch_bacon_file(file_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        //  let mut watcher: RecommendedWatcher = Watcher::new(tx, notify::Config::default())?;
            let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(Path::new(file_path), RecursiveMode::NonRecursive)?;


        println!("Watching {} for changes...", file_path);

        let mut first_modify: Option<Instant> = None;
        let mut save_count: usize = 0;
        let overall_start = Instant::now();
        let timeout = Duration::from_secs(10);

        loop {
            println!("Waiting for event...");
            // Wait up to 10 seconds for an event.
            match rx.recv_timeout(Duration::from_secs(4)) {
                Ok(event) => {
                    println!("Received event: {:?}", event);
                    if let EventKind::Modify(_) = event.unwrap().kind {
                        let now = Instant::now();
                        if let Some(first) = first_modify {
                            if now.duration_since(first) > timeout {
                                // Too long since the first modification; reset counter.
                                first_modify = Some(now);
                                save_count = 1;
                                println!("Modification event received. Resetting count. Save count: {}", save_count);
                            } else {
                                save_count += 1;
                                println!("Modification event received. Save count: {}", save_count);
                                if save_count >= 2 {
                                    println!("Double modification detected within {} seconds.", timeout.as_secs()); 
                                    return Ok(true);
                                }
                            }
                        } else {
                            first_modify = Some(now);
                            save_count = 1;
                            println!("Modification event received. Save count: {}", save_count);
                        }
                    }
                },
                Err(_) => {
                    println!("No further modifications detected within 10 seconds.");
                    return Ok(false);
                }
            }
            if overall_start.elapsed() > Duration::from_secs(12) {
                return Ok(false);
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    #[cfg(feature = "uses_notify")]
pub(crate) async fn monitor_linerefs(bacon_file: &str) {
    use tokio::time::sleep;
    use std::time::Duration;

    println!("Monitoring bacon file for warnings. Press Ctrl+C to exit.");
    loop {
        match crate::read_bacon_file(bacon_file) {
            Ok(targets) => {
                if !targets.is_empty() {
                    let target = &targets[0];
                    match crate::should_open_lineref(target) {
                        Ok(true) => {
                            if let Err(e) = crate::open_line_ref(target).await {
                                eprintln!("Error opening warning: {}", e);
                            } else {
                                println!("Editor closed. Re-reading bacon file for any updates...");
                            }
                        }
                        Ok(false) => {
                        }
                        Err(e) => {
                            eprintln!("Error checking warning {}: {}", target, e);
                        }
                    }
                } else {
                    println!("No warnings found. Waiting for updates...");
                    sleep(Duration::from_secs(2)).await;
                }
            }
            Err(e) => {
                eprintln!("Error reading bacon file: {}", e);
            }
        }
                sleep(Duration::from_secs(2)).await;
    }
}

}



/// Checks if a warning target should be opened based on the current file contents.
/// The target is expected in the form "file_path:line:column".
/// If this target is not yet recorded, its line content is read and stored and the function returns true.
/// Otherwise, if it is recorded and the current line is different (or if it was already processed),
/// it returns false.
/// 
/// This function is now called `should_open_lineref`.
/// 
fn should_open_lineref(target: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Find the last colon (separating column)
    if let Some(last_colon) = target.rfind(':') {
        // Find the second-to-last colon (separating line number)
        if let Some(second_last_colon) = target[..last_colon].rfind(':') {
            let file_path = &target[..second_last_colon];
            let line_str = &target[second_last_colon + 1..last_colon];
            // Parse the line number (this should now work correctly even with Windows paths)
            let line_number: usize = line_str.parse()?;
            
            // Read the file's content and split into lines
            let content = fs::read_to_string(file_path)?;
            let lines: Vec<&str> = content.lines().collect();
            if line_number == 0 || line_number > lines.len() {
                return Err("Line number out of bounds".into());
            }
            let current_line = lines[line_number - 1].to_string();
            
            let mut map = LINES_OF_CONCERN.lock().unwrap();
            if map.contains_key(target) {
                // Already processed; return false without printing additional messages.
                Ok(false)
            } else {
                // Record this target for the first time and allow it to be processed.
                map.insert(target.to_string(), current_line);
                Ok(true)
            }
        } else {
            Err("Target format error: could not find second colon".into())
        }
    } else {
        Err("Target format error: could not find colon".into())
    }
}



// #[cfg(feature = "uses_notify")]
// mod notify_watch {
//     use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
//     use std::sync::mpsc::channel;
//     use std::time::{Duration, Instant};
//     use std::path::Path;

//     /// Watches the given file for modifications.
//     ///
//     /// If the file is modified twice within 10 seconds, returns Ok(true).
//     /// Otherwise, if no double-modification is detected within 10 seconds, returns Ok(false).
//     pub fn watch_bacon_file(file_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
//         let (tx, rx) = channel();
//         // Create a debounced watcher with a 1‑second delay.
//         let mut watcher: RecommendedWatcher = Watcher::new(tx, notify::Config::default())?;
//         watcher.watch(Path::new(file_path), RecursiveMode::NonRecursive)?;
//         println!("Watching {} for changes...", file_path);

//         let mut first_modify: Option<Instant> = None;
//         let overall_start = Instant::now();
//         let timeout = Duration::from_secs(10);
        
//         loop {
//             // Wait up to 10 seconds for an event.
//             match rx.recv_timeout(Duration::from_secs(10)) {
//                 Ok(event) => {
//                     // Check if the event is a Modify event.
//                     if let EventKind::Modify(_) = event.unwrap().kind {
//                         let now = Instant::now();
//                         if let Some(first) = first_modify {
//                             if now.duration_since(first) <= timeout {
//                                 println!("Double modification detected within 10 seconds.");
//                                 return Ok(true);
//                             } else {
//                                 // Reset the first modify time if outside timeout.
//                                 first_modify = Some(now);
//                             }
//                         } else {
//                             first_modify = Some(now);
//                         }
//                     }
//                 },
//                 Err(_timeout_err) => {
//                     // No event received within timeout.
//                     println!("No further modifications detected within 10 seconds.");
//                     return Ok(false);
//                 }
//             }
//             if overall_start.elapsed() > Duration::from_secs(12) {
//                 return Ok(false);
//             }
//         }
//     }
// }

#[cfg(all(test, feature = "uses_notify"))]
mod tests_notify {
    use std::fs::{self, File};
    use std::io::Write;
    use std::thread;
    use std::time::Duration;
    // Import the watch function from our notify module.
    use crate::notify_watch::watch_bacon_file;

    #[test]
    fn test_watch_bacon_file_double_save() {
        // Create a temporary "testgen" folder.
        let testgen_dir = "testgen";
        fs::create_dir_all(testgen_dir).unwrap();

        let test_file = format!("{}/test_bacon_notify.txt", testgen_dir);
        // Write initial content.
        {
            let mut file = File::create(&test_file).unwrap();
            writeln!(file, "Initial content").unwrap();
        }
        
        // Spawn a thread to watch the file.
        let handle = thread::spawn({
            let test_file = test_file.clone();
            move || {
                // This will block until a double modification is detected or times out.
                watch_bacon_file(&test_file).unwrap()
            }
        });

        // Wait a moment to ensure the watcher is set up.
        thread::sleep(Duration::from_secs(2));

        // Simulate a first save (modify the file).
        {
            let mut file = File::create(&test_file).unwrap();
            writeln!(file, "Modified content 1").unwrap();
        }
        thread::sleep(Duration::from_secs(2));

        // Simulate a second save within 10 seconds.
        {
            let mut file = File::create(&test_file).unwrap();
            writeln!(file, "Modified content 2").unwrap();
        }

        let result = handle.join().unwrap();
        assert!(result, "Expected double save detection to return true");

        // Cleanup.
        fs::remove_file(&test_file).unwrap();
        fs::remove_dir(testgen_dir).unwrap();
    }

    #[test]
    fn test_watch_bacon_file_normal_save() {
        // Create a temporary "testgen" folder.
        let testgen_dir = "testgen";
        fs::create_dir_all(testgen_dir).unwrap();

        let test_file = format!("{}/test_bacon_notify_normal.txt", testgen_dir);
        // Write initial content.
        {
            let mut file = File::create(&test_file).unwrap();
            writeln!(file, "Initial content").unwrap();
        }
        
        // Spawn the watcher in a separate thread.
        let handle = thread::spawn({
            let test_file = test_file.clone();
            move || {
                watch_bacon_file(&test_file).unwrap()
            }
        });
        
        // Wait a moment to ensure the watcher is set up.
        thread::sleep(Duration::from_secs(2));
        
        // Simulate a single save (modify the file once).
        {
            let mut file = File::create(&test_file).unwrap();
            writeln!(file, "Modified content").unwrap();
        }
        
        // Do not trigger a second save within 10 seconds; wait for the watcher to time out.
        let result = handle.join().unwrap();
        assert!(!result, "Expected single save to return false, indicating normal save behavior");
        
        // Cleanup.
        fs::remove_file(&test_file).unwrap();
        fs::remove_dir(testgen_dir).unwrap();
    }
}
