// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

pub use std::io::{self, Write};
pub use textwrap::Options as Textwrap_Options;
pub use textwrap::fill as Textwrap_Fill;

/// Log levels for CLI output.
///
/// Defines the different types of messages that can be logged, from simple
/// output to debug and trace messages.
#[derive(Debug, Clone, Copy)]
pub enum CliLevel {
    /// Send output without newline.
    Send,
    /// Send output with newline.
    SendLn,
    /// Informational message.
    Info,
    /// Warning message.
    Warn,
    /// Error message.
    Error,
    /// Debug message (requires `log` feature).
    Debug,
    /// Trace message (requires `log` feature).
    Trace,
}

/// Core logging function used by CLI output macros.
///
/// Formats the text with provided arguments, applies word wrapping, and outputs
/// according to the specified log level. When the `log` feature is enabled, also
/// logs to the configured logging backend.
///
/// # Arguments
///
/// * `level` - The log level determining output behavior
/// * `text` - The text to output (may contain `{}` placeholders)
/// * `args` - Arguments to replace placeholders in the text
pub fn cli_log(level: CliLevel, text: &str, args: &[String]) {
    let wrapped = format_wrapped(text, args, None);

    match level {
        CliLevel::Send => {
            print!("{}", wrapped);
        }
        CliLevel::SendLn => {
            println!("{}", wrapped);
        }

        _other => {
            #[cfg(feature = "log")]
            match other {
                CliLevel::Info => log::info!("{}", text),
                CliLevel::Warn => log::warn!("{}", text),
                CliLevel::Error => log::error!("{}", text),
                CliLevel::Debug => log::debug!("{}", text),
                CliLevel::Trace => log::trace!("{}", text),
                _ => {}
            }
            println!("{}", wrapped);
        }
    }

    io::stdout().flush().unwrap();
}

/// Formats text with argument substitution and word wrapping.
///
/// Replaces `{}` placeholders in the text with arguments, optionally adds a prefix,
/// and applies word wrapping at 75 characters.
///
/// # Arguments
///
/// * `text` - The text to format (may contain `{}` placeholders)
/// * `args` - Arguments to replace placeholders
/// * `prefix` - Optional prefix to prepend to the text
fn format_wrapped(text: &str, args: &[String], prefix: Option<&str>) -> String {
    // Replace placeholders
    let mut text = text.to_string();
    for arg in args {
        text = text.replacen("{}", arg, 1);
    }

    if let Some(prefix) = prefix {
        text = format!("{}{}", prefix, text);
    }

    // Word wrap
    Textwrap_Fill(&text, Textwrap_Options::new(75))
}

/// Outputs text without a newline.
///
/// Similar to `print!`, but with word wrapping. Supports format-style arguments.
///
/// # Example
///
/// ```
/// use falcon_cli::cli_send;
///
/// cli_send!("Loading");
/// cli_send!("...");  // Continues on same line
/// ```
#[macro_export]
macro_rules! cli_send {
    ($text:expr) => { $crate::cli_log($crate::CliLevel::Send, $text, &[]) };
    ($text:expr, $( $arg:expr ),*) => {{
        let mut args = vec![];
        $( args.push($arg.to_string()); )*
        $crate::cli_log($crate::CliLevel::Send, $text, &args)
    }};
}

/// Outputs text with a newline.
///
/// Similar to `println!`, but with word wrapping. Supports format-style arguments.
///
/// # Example
///
/// ```
/// use falcon_cli::cli_sendln;
///
/// cli_sendln!("Hello, world!");
/// cli_sendln!("User: {}", "Alice");
/// ```
#[macro_export]
macro_rules! cli_sendln {
    ($text:expr) => { $crate::cli_log($crate::CliLevel::SendLn, $text, &[]) };
    ($text:expr, $( $arg:expr ),*) => {{
        let mut args = vec![];
        $( args.push($arg.to_string()); )*
        $crate::cli_log($crate::CliLevel::SendLn, $text, &args)
    }};
}

/// Outputs an informational message.
///
/// Displays text and optionally logs to the configured logger when the `log` feature is enabled.
///
/// # Example
///
/// ```
/// use falcon_cli::cli_info;
///
/// cli_info!("Application started successfully");
/// cli_info!("Loaded {} configuration files", 5);
/// ```
#[macro_export]
macro_rules! cli_info {
    ($text:expr) => { $crate::cli_log($crate::CliLevel::Info, $text, &[]) };
    ($text:expr, $( $arg:expr ),*) => {{
        let mut args = vec![];
        $( args.push($arg.to_string()); )*
        $crate::cli_log($crate::CliLevel::Info, $text, &args)
    }};
}

/// Outputs a warning message.
///
/// Displays text and optionally logs to the configured logger when the `log` feature is enabled.
///
/// # Example
///
/// ```
/// use falcon_cli::cli_warn;
///
/// cli_warn!("Configuration file not found, using defaults");
/// cli_warn!("Deprecated feature: {}", "old_api");
/// ```
#[macro_export]
macro_rules! cli_warn {
    ($text:expr) => { $crate::cli_log($crate::CliLevel::Warn, $text, &[]) };
    ($text:expr, $( $arg:expr ),*) => {{
        let mut args = vec![];
        $( args.push($arg.to_string()); )*
        $crate::cli_log($crate::CliLevel::Warn, $text, &args)
    }};
}

/// Outputs an error message.
///
/// Displays text and optionally logs to the configured logger when the `log` feature is enabled.
///
/// # Example
///
/// ```
/// use falcon_cli::cli_error;
///
/// cli_error!("Failed to connect to database");
/// cli_error!("Invalid input: {}", input_value);
/// ```
#[macro_export]
macro_rules! cli_error {
    ($text:expr) => { $crate::cli_log($crate::CliLevel::Error, $text, &[]) };
    ($text:expr, $( $arg:expr ),*) => {{
        let mut args = vec![];
        $( args.push($arg.to_string()); )*
        $crate::cli_log($crate::CliLevel::Error, $text, &args)
    }};
}

/// Outputs a debug message.
///
/// Displays text and logs to the configured logger when the `log` feature is enabled.
///
/// # Example
///
/// ```
/// use falcon_cli::cli_debug;
///
/// cli_debug!("Processing step 1 of 3");
/// cli_debug!("Variable value: {}", debug_value);
/// ```
#[macro_export]
macro_rules! cli_debug {
    ($text:expr) => { $crate::cli_log($crate::CliLevel::Debug, $text, &[]) };
    ($text:expr, $( $arg:expr ),*) => {{
        let mut args = vec![];
        $( args.push($arg.to_string()); )*
        $crate::cli_log($crate::CliLevel::Debug, $text, &args)
    }};
}

/// Outputs a trace message.
///
/// Displays text and logs to the configured logger when the `log` feature is enabled.
/// Used for very detailed diagnostic information.
///
/// # Example
///
/// ```
/// use falcon_cli::cli_trace;
///
/// cli_trace!("Entering function parse_config");
/// cli_trace!("Loop iteration: {}", i);
/// ```
#[macro_export]
macro_rules! cli_trace {
    ($text:expr) => { $crate::cli_log($crate::CliLevel::Trace, $text, &[]) };
    ($text:expr, $( $arg:expr ),*) => {{
        let mut args = vec![];
        $( args.push($arg.to_string()); )*
        $crate::cli_log($crate::CliLevel::Trace, $text, &args)
    }};
}
