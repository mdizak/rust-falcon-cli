//#![allow(warnings)]
// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

pub use self::error::CliError;
pub use self::help::CliHelpScreen;
pub use self::macros::*;
pub use self::request::{CliFormat, CliRequest};
pub use self::router::CliRouter;
pub use anyhow;
pub use indexmap::{IndexMap, indexmap};

use rpassword::read_password;
use std::fmt::Display;
use std::hash::Hash;
use std::process::{Command, exit};
use std::str::FromStr;
use std::{env, fs};
use zxcvbn::zxcvbn;

pub mod error;
mod help;
pub mod macros;
mod request;
mod router;

/// Trait that all CLI commands must implement.
///
/// This trait defines the interface for CLI commands, requiring implementations
/// to provide both a `process` method for executing the command and a `help` method
/// for generating help documentation.
///
/// # Example
///
/// ```
/// use falcon_cli::{CliCommand, CliRequest, CliHelpScreen};
///
/// struct MyCommand;
///
/// impl CliCommand for MyCommand {
///     fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
///         println!("Executing command");
///         Ok(())
///     }
///
///     fn help(&self) -> CliHelpScreen {
///         CliHelpScreen::new("My Command", "myapp mycommand", "Does something useful")
///     }
/// }
/// ```
pub trait CliCommand {
    /// Processes the CLI command with the given request.
    ///
    /// # Arguments
    ///
    /// * `req` - The CLI request containing arguments, flags, and other parsed data
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an error on failure.
    fn process(&self, req: &CliRequest) -> anyhow::Result<()>;

    /// Returns the help screen for this command.
    ///
    /// This method should create and return a `CliHelpScreen` with information
    /// about how to use the command, including parameters, flags, and examples.
    fn help(&self) -> CliHelpScreen;
}

/// Executes the CLI command router and processes the appropriate command.
///
/// This function should be called once all necessary routes have been defined via
/// `router.add()`. It will parse command line arguments, look up the appropriate
/// command handler, and execute it or display help as needed.
///
/// # Arguments
///
/// * `router` - A mutable reference to the configured CLI router
///
/// # Example
///
/// ```no_run
/// use falcon_cli::{CliRouter, cli_run};
///
/// let mut router = CliRouter::new();
/// router.app_name("My App");
/// // Add commands here...
/// cli_run(&mut router);
/// ```
pub fn cli_run(router: &mut CliRouter) {
    // Lookup route
    let (req, cmd) = match router.lookup() {
        Some(r) => r,
        None => {
            CliHelpScreen::render_index(&router);
            exit(0);
        }
    };

    // Process as needed
    if req.is_help {
        CliHelpScreen::render(&cmd, &req.cmd_alias, &req.shortcuts);
    } else if let Err(e) = cmd.process(&req) {
        cli_send!("ERROR: {}\n", e);
    }
}

/// Displays a formatted header in the terminal.
///
/// Outputs the given text with 30 dashes at the top and bottom to create a header section.
///
/// # Arguments
///
/// * `text` - The text to display in the header
///
/// # Example
///
/// ```
/// use falcon_cli::cli_header;
///
/// cli_header("My Application");
/// // Output:
/// // ------------------------------
/// // -- My Application
/// // ------------------------------
/// ```
pub fn cli_header(text: &str) {
    println!("------------------------------");
    println!("-- {}", text);
    println!("------------------------------\n");
}

/// Prompts the user to select an option from a list.
///
/// Displays a question and list of options, then waits for the user to select one.
/// The function will continue prompting until a valid option is selected.
///
/// # Arguments
///
/// * `question` - The question or prompt to display
/// * `options` - An `IndexMap` of options where keys are option identifiers and values are descriptions
///
/// # Returns
///
/// Returns the key of the selected option.
///
/// # Example
///
/// ```no_run
/// use falcon_cli::{cli_get_option, indexmap};
/// use indexmap::IndexMap;
///
/// let options = indexmap! {
///     1 => "First option",
///     2 => "Second option",
///     3 => "Third option",
/// };
///
/// let selected = cli_get_option("Which option do you prefer?", &options);
/// println!("You selected: {}", selected);
/// ```
pub fn cli_get_option<K, V>(question: &str, options: &IndexMap<K, V>) -> K
where
    K: Display + Eq + PartialEq + Hash + FromStr,
    <K as FromStr>::Err: Display,
    V: Display,
{
    let message = format!("{}\n\n", question);
    cli_send!(&message);
    for (key, value) in options.iter() {
        cli_send!(&format!("    [{}] {}\n", key, value));
    }
    cli_send!("\nSelect One: ");

    // Get user input
    let mut input: String;
    loop {
        input = String::new();

        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();

        if let Ok(value) = input.parse::<K>() {
            if options.contains_key(&value) {
                return value;
            }
        }

        print!("\r\nInvalid option, try again: ");
        io::stdout().flush().unwrap();
    }
}

/// Gets text input from the user.
///
/// Displays a prompt message and waits for the user to enter text. If the user
/// enters nothing, the default value is returned.
///
/// # Arguments
///
/// * `message` - The prompt message to display
/// * `default_value` - The value to return if the user enters nothing
///
/// # Returns
///
/// Returns the user's input as a `String`, or the default value if no input was provided.
///
/// # Example
///
/// ```no_run
/// use falcon_cli::cli_get_input;
///
/// let name = cli_get_input("Enter your name: ", "Anonymous");
/// println!("Hello, {}!", name);
/// ```
pub fn cli_get_input(message: &str, default_value: &str) -> String {
    // Display message
    cli_send!(message);
    io::stdout().flush().unwrap();

    // Get user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let mut input = input.trim();

    // Default value, if needed
    if input.trim().is_empty() {
        input = default_value;
    }

    input.to_string()
}

/// Gets multi-line text input from the user.
///
/// Displays a prompt message and collects multiple lines of input from the user.
/// Input collection stops when the user enters an empty line.
///
/// # Arguments
///
/// * `message` - The prompt message to display
///
/// # Returns
///
/// Returns all entered lines joined with newline characters as a single `String`.
///
/// # Example
///
/// ```no_run
/// use falcon_cli::cli_get_multiline_input;
///
/// let description = cli_get_multiline_input("Enter description:");
/// println!("You entered:\n{}", description);
/// ```
pub fn cli_get_multiline_input(message: &str) -> String {
    // Display message
    cli_send!(&format!("{} (empty line to stop)\n\n", message));
    io::stdout().flush().unwrap();

    // Get user input
    let mut res: Vec<String> = Vec::new();
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();

        if input.is_empty() {
            break;
        }
        res.push(input.to_string());
    }

    res.join("\n").to_string()
}

/// Requests confirmation from the user.
///
/// Displays a message and prompts the user to answer yes (y) or no (n).
/// The function will continue prompting until a valid response is received.
///
/// # Arguments
///
/// * `message` - The confirmation message to display
///
/// # Returns
///
/// Returns `true` if the user answered 'y', `false` if they answered 'n'.
///
/// # Example
///
/// ```no_run
/// use falcon_cli::cli_confirm;
///
/// if cli_confirm("Do you want to continue?") {
///     println!("Continuing...");
/// } else {
///     println!("Cancelled.");
/// }
/// ```
pub fn cli_confirm(message: &str) -> bool {
    // Send message
    let confirm_message = format!("{} (y/n): ", message);
    cli_send!(&confirm_message);

    // Get user input
    let mut _input = "".to_string();
    loop {
        _input = String::new();

        io::stdin().read_line(&mut _input).expect("Failed to read line");
        let _input = _input.trim().to_lowercase();

        if _input != "y" && _input != "n" {
            cli_send!("Invalid option, please try again.  Enter (y/n): ");
        } else {
            break;
        }
    }

    // Return
    let res_char = _input.chars().next().unwrap();

    res_char == 'y'
}

/// Gets a password from the user without displaying the input on screen.
///
/// Prompts the user for a password with input hidden from the terminal.
/// Optionally can require a non-empty password.
///
/// # Arguments
///
/// * `message` - The prompt message to display (defaults to "Password: " if empty)
/// * `allow_blank` - Whether to allow an empty password
///
/// # Returns
///
/// Returns the entered password as a `String`.
///
/// # Example
///
/// ```no_run
/// use falcon_cli::cli_get_password;
///
/// let password = cli_get_password("Enter password: ", false);
/// println!("Password entered successfully");
/// ```
pub fn cli_get_password(message: &str, allow_blank: bool) -> String {
    // Get message
    let password_message = if message.is_empty() {
        "Password: "
    } else {
        message
    };

    // Get password
    let mut _password = String::new();
    loop {
        cli_send!(password_message);
        _password = read_password().unwrap();

        if _password.is_empty() && !allow_blank {
            cli_send!("You did not specify a password");
        } else {
            break;
        }
    }

    _password
}

/// Gets a new password from the user with confirmation and strength validation.
///
/// Prompts the user to enter a password twice for confirmation and validates it
/// against a required strength level using the zxcvbn algorithm. The function will
/// continue prompting until a password meeting all requirements is entered.
///
/// # Arguments
///
/// * `req_strength` - Required password strength (0-4, where 4 is strongest)
///   - 0: Too guessable
///   - 1: Very guessable
///   - 2: Somewhat guessable
///   - 3: Safely unguessable
///   - 4: Very unguessable
///
/// # Returns
///
/// Returns the validated password as a `String`.
///
/// # Example
///
/// ```no_run
/// use falcon_cli::cli_get_new_password;
///
/// // Require a password with strength level 3
/// let password = cli_get_new_password(3);
/// println!("Strong password created successfully");
/// ```
pub fn cli_get_new_password(req_strength: u8) -> String {
    // Initialize
    let mut _password = String::new();
    let mut _confirm_password = String::new();

    // Get new password
    loop {
        cli_send!("Desired Password: ");
        _password = read_password().unwrap();

        if _password.is_empty() {
            cli_send!("You did not specify a password");
            continue;
        }

        // Check strength
        let strength = zxcvbn(&_password, &[]).unwrap();
        if strength.score() < req_strength {
            cli_send!("Password is not strong enough.  Please try again.\n\n");
            continue;
        }

        // Confirm password
        cli_send!("Confirm Password: ");
        _confirm_password = read_password().unwrap();
        if _password != _confirm_password {
            cli_send!("Passwords do not match, please try again.\n\n");
            continue;
        }
        break;
    }

    _password
}

/// Displays data in a formatted table.
///
/// Renders data in a tabular format similar to SQL database output, with borders
/// and properly aligned columns. Column widths are automatically calculated based
/// on the content.
///
/// # Arguments
///
/// * `columns` - Slice of column headers
/// * `rows` - Slice of rows, where each row is a vector of cell values
///
/// # Example
///
/// ```no_run
/// use falcon_cli::cli_display_table;
///
/// let columns = ["Name", "Age", "City"];
/// let rows = vec![
///     vec!["Alice", "30", "New York"],
///     vec!["Bob", "25", "London"],
///     vec!["Charlie", "35", "Tokyo"],
/// ];
///
/// cli_display_table(&columns, &rows);
/// ```
pub fn cli_display_table<C: Display, R: Display>(columns: &[C], rows: &[Vec<R>]) {
    // Return if no rows
    if rows.is_empty() {
        println!("No rows to display.\n");
        return;
    }

    // Initialize sizes - using index-based approach since we can't use T as HashMap key
    let mut sizes: Vec<usize> = vec![0; columns.len()];

    // Get sizes of column headers
    for (i, col) in columns.iter().enumerate() {
        let col_str = col.to_string();
        sizes[i] = col_str.len();
    }

    // Get maximum sizes by checking all row values
    for row in rows {
        for (i, val) in row.iter().enumerate() {
            if i < sizes.len() {
                let val_str = val.to_string();
                let val_len = val_str.len();
                if val_len > sizes[i] {
                    sizes[i] = val_len;
                }
            }
        }
    }

    // Add padding to all column sizes
    for size in sizes.iter_mut() {
        *size += 3;
    }

    // Initialize header variables
    let mut header = String::from("+");
    let mut col_header = String::from("|");

    // Print column headers
    for (i, col) in columns.iter().enumerate() {
        let col_str = col.to_string();
        let padded_col = format!("{}{}", col_str, " ".repeat(sizes[i] - col_str.len()));
        header += &("-".repeat(sizes[i] + 1) + "+");
        col_header += &format!(" {}|", padded_col);
    }

    println!("{}\n{}\n{}", header, col_header, header);

    // Display the rows
    for row in rows {
        let mut line = String::from("|");
        for (i, val) in row.iter().enumerate() {
            if i < sizes.len() {
                let val_str = val.to_string();
                let padded_val = format!(" {}{}", val_str, " ".repeat(sizes[i] - val_str.len()));
                line += &format!("{}|", padded_val);
            }
        }
        println!("{}", line);
    }
    println!("{}\n", header);
}

/// Displays a two-column array with proper spacing and word wrapping.
///
/// Formats and displays key-value pairs in two columns with automatic text wrapping.
/// This function is primarily used by the help system to display parameters and flags,
/// but can be used for any two-column data display.
///
/// # Arguments
///
/// * `rows` - An `IndexMap` where keys are displayed in the left column and values in the right
///
/// # Example
///
/// ```no_run
/// use falcon_cli::{cli_display_array, indexmap};
/// use indexmap::IndexMap;
///
/// let mut items = indexmap! {
///     "--verbose" => "Enable verbose output with detailed logging",
///     "--output" => "Specify the output file path",
///     "--help" => "Display this help message",
/// };
///
/// cli_display_array(&items);
/// ```
pub fn cli_display_array<K: Display, V: Display>(rows: &IndexMap<K, V>) {
    // Get max left column size
    let mut size = 0;
    for key in rows.keys() {
        let key_str = key.to_string();
        if key_str.len() + 8 > size {
            size = key_str.len() + 8;
        }
    }
    let indent = " ".repeat(size);
    let indent_size = size - 4;

    // Go through rows
    for (key, value) in rows {
        let key_str = key.to_string();
        let value_str = value.to_string();
        let left_col = format!("    {}{}", key_str, " ".repeat(indent_size - key_str.len()));
        let options =
            textwrap::Options::new(75).initial_indent(&left_col).subsequent_indent(&indent);
        let line = textwrap::fill(&value_str, &options);
        println!("{}", line);
    }
    println!("");
}

/// Clears the terminal screen.
///
/// Sends the ANSI escape sequence to clear all lines and reset the cursor position.
///
/// # Example
///
/// ```no_run
/// use falcon_cli::cli_clear_screen;
///
/// cli_clear_screen();
/// println!("Screen cleared!");
/// ```
pub fn cli_clear_screen() {
    print!("\x1B[2J");
}

/// Opens a text editor for the user to edit content.
///
/// Creates a temporary file with the provided contents, opens it in the user's
/// preferred text editor, and returns the edited content. The editor used is
/// determined by the `VISUAL` or `EDITOR` environment variables, with sensible
/// defaults for each platform.
///
/// # Arguments
///
/// * `contents` - The initial content to populate the editor with
///
/// # Returns
///
/// Returns `Ok(String)` with the edited content on success, or a `CliError` if
/// the editor fails to launch or exits with an error.
///
/// # Example
///
/// ```no_run
/// use falcon_cli::cli_text_editor;
///
/// let initial = "Edit this text...";
/// match cli_text_editor(initial) {
///     Ok(edited) => println!("New content: {}", edited),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn cli_text_editor(contents: &str) -> Result<String, CliError> {
    // Create temp file
    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "cli_edit_{}.tmp",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
    ));

    // Write initial contents to temp file
    fs::write(&temp_file, contents)
        .map_err(|e| CliError::Generic(format!("Failed to create temp file: {}", e)))?;

    // Get editor command
    let editor = get_editor();

    // Launch editor
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", &format!("{} \"{}\"", editor, temp_file.display())])
            .status()
    } else {
        Command::new(&editor).arg(&temp_file).status()
    };

    match status {
        Ok(exit_status) if exit_status.success() => {
            // Read the file contents
            let result = fs::read_to_string(&temp_file).unwrap_or_else(|_| String::new());

            // Delete temp file
            let _ = fs::remove_file(&temp_file);

            Ok(result)
        }
        Ok(_) => {
            let _ = fs::remove_file(&temp_file);
            Err(CliError::Generic("Editor exited with error".to_string()))
        }
        Err(e) => {
            let _ = fs::remove_file(&temp_file);
            Err(CliError::Generic(format!("Failed to launch editor: {}", e)))
        }
    }
}

/// Determines the text editor to use based on environment variables and platform.
///
/// Checks environment variables in order of preference (`VISUAL`, then `EDITOR`),
/// falling back to platform-specific defaults if neither is set.
fn get_editor() -> String {
    // Check environment variables in order of preference
    if let Ok(editor) = env::var("VISUAL") {
        return editor;
    }
    if let Ok(editor) = env::var("EDITOR") {
        return editor;
    }

    // Platform-specific defaults
    if cfg!(target_os = "windows") {
        // Try notepad++ first, fall back to notepad
        if Command::new("notepad++").arg("--version").output().is_ok() {
            "notepad++".to_string()
        } else {
            "notepad".to_string()
        }
    } else if cfg!(target_os = "macos") {
        // macOS - try nano first (comes default), then vim
        if Command::new("which").arg("nano").output().is_ok() {
            "nano".to_string()
        } else {
            "vim".to_string()
        }
    } else {
        // Linux/Unix - try in order of user-friendliness
        for editor in &["nano", "vim", "vi"] {
            if Command::new("which")
                .arg(editor)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return editor.to_string();
            }
        }
        "vi".to_string() // Last resort, should always exist on Unix
    }
}

/// Creates and displays a new progress bar.
///
/// Initializes a progress bar with the specified message and total value,
/// and immediately renders it at 0% completion.
///
/// # Arguments
///
/// * `message` - The message to display alongside the progress bar
/// * `total` - The total value representing 100% completion
///
/// # Returns
///
/// Returns a `CliProgressBar` instance that can be updated with `increment()` or `set()`.
///
/// # Example
///
/// ```no_run
/// use falcon_cli::cli_progress_bar;
///
/// let mut bar = cli_progress_bar("Processing files", 100);
/// for i in 0..100 {
///     // Do work...
///     bar.increment(1);
/// }
/// bar.finish();
/// ```
pub fn cli_progress_bar(message: &str, total: usize) -> CliProgressBar {
    let bar = CliProgressBar {
        value: 0,
        total,
        message: message.to_string(),
    };
    bar.start();
    bar
}

/// A progress bar for displaying task completion in the terminal.
///
/// This struct maintains the state of a progress bar and provides methods
/// to update and render it. The bar displays percentage, a message, and
/// a visual indicator of progress.
pub struct CliProgressBar {
    /// Current value of progress (0 to total).
    pub value: usize,
    /// Total value representing 100% completion.
    pub total: usize,
    /// Message displayed alongside the progress bar.
    pub message: String,
}

impl CliProgressBar {
    /// Initializes and displays the progress bar.
    ///
    /// Renders the progress bar on a new line at its initial state (0%).
    pub fn start(&self) {
        self.render();
    }

    /// Increments the progress value and updates the display.
    ///
    /// # Arguments
    ///
    /// * `num` - The amount to increment the progress by
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::cli_progress_bar;
    /// let mut bar = cli_progress_bar("Processing", 100);
    /// bar.increment(10); // Progress is now at 10%
    /// bar.increment(15); // Progress is now at 25%
    /// ```
    pub fn increment(&mut self, num: usize) {
        self.value = self.value.saturating_add(num).min(self.total);
        self.render();
    }

    /// Sets the progress to a specific value and updates the display.
    ///
    /// # Arguments
    ///
    /// * `value` - The new progress value (clamped to `total`)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::cli_progress_bar;
    /// let mut bar = cli_progress_bar("Processing", 100);
    /// bar.set(50); // Set progress to 50%
    /// ```
    pub fn set(&mut self, value: usize) {
        self.value = value.min(self.total);
        self.render();
    }

    /// Completes the progress bar.
    ///
    /// Sets the progress to 100%, renders the final state, and moves to a new line.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::cli_progress_bar;
    /// let mut bar = cli_progress_bar("Processing", 100);
    /// // ... do work ...
    /// bar.finish();
    /// ```
    pub fn finish(&mut self) {
        self.value = self.total;
        self.render();
        println!("");
    }

    /// Renders the progress bar to the terminal.
    ///
    /// Internal method that calculates and displays the progress bar with
    /// percentage, message, and visual indicator.
    fn render(&self) {
        let percent = if self.total > 0 {
            (self.value * 100) / self.total
        } else {
            0
        };

        // Calculate available space
        // Format: [ xx% ] <MESSAGE> [******      ]
        // Fixed parts: "[ ", "% ] ", " [", "]" = 8 chars
        // Percent: 1-3 chars (0-100)
        let percent_str = format!("{}", percent);
        let fixed_overhead = 8 + percent_str.len();

        // Available space for message and bar
        let available = 75_usize.saturating_sub(fixed_overhead);

        // Reserve minimum 10 chars for bar (including brackets)
        let bar_size = 10;
        let message_max = available.saturating_sub(bar_size);

        // Truncate message if needed
        let display_message = if self.message.len() > message_max {
            format!("{}...", &self.message[..message_max.saturating_sub(3)])
        } else {
            self.message.clone()
        };

        // Calculate actual bar width (inner width without brackets)
        let bar_width = available.saturating_sub(display_message.len()).max(8);
        let filled = (bar_width * self.value) / self.total.max(1);
        let empty = bar_width.saturating_sub(filled);

        // Build the bar
        let bar = format!("{}{}", "*".repeat(filled), " ".repeat(empty));

        // Print with carriage return to overwrite line
        print!("\r[ {}% ] {} [{}]", percent, display_message, bar);
        io::stdout().flush().unwrap();

        // Print newline when complete
        if self.value >= self.total {
            println!();
        }
    }
}
