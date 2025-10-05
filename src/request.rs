// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::error::CliError;
use std::collections::HashMap;
use std::fs;
use std::ops::Range;
use url::Url;

/// Represents a parsed CLI command request.
///
/// This struct contains all the parsed information from a command line invocation,
/// including the command name, arguments, flags, and their values.
pub struct CliRequest {
    /// The primary alias of the command that was invoked.
    pub cmd_alias: String,
    /// Whether help was requested for this command.
    pub is_help: bool,
    /// Positional arguments passed to the command.
    pub args: Vec<String>,
    /// Boolean flags that were provided (e.g., `-v`, `--verbose`).
    pub flags: Vec<String>,
    /// Flags with associated values (e.g., `--output file.txt`).
    pub flag_values: HashMap<String, String>,
    /// List of shortcut aliases for this command.
    pub shortcuts: Vec<String>,
}

/// Format validators for command arguments and flags.
///
/// These validators can be used to ensure arguments and flags conform to
/// expected formats before processing.
#[derive(Clone, PartialEq)]
pub enum CliFormat {
    /// Accept any string value.
    Any,
    /// Must be a valid integer.
    Integer,
    /// Must be a valid decimal number.
    Decimal,
    /// Must be a boolean value (true/false, yes/no, 1/0).
    Boolean,
    /// Must be a valid email address.
    Email,
    /// Must be a valid URL.
    Url,
    /// String length must be within the specified range.
    StringRange(Range<usize>),
    /// Integer value must be within the specified range.
    IntegerRange(Range<i64>),
    /// Decimal value must be within the specified range.
    DecimalRange(Range<f64>),
    /// Value must be one of the specified options.
    OneOf(Vec<String>),
    /// Must be a path to an existing file.
    File,
    /// Must be a path to an existing directory.
    Directory,
}

impl CliRequest {
    /// Ensures that at least the specified number of parameters were provided.
    ///
    /// # Arguments
    ///
    /// * `num` - The minimum number of required parameters
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if enough parameters were provided, or `CliError::MissingParams` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::{CliRequest, CliCommand, CliHelpScreen};
    /// # struct MyCmd;
    /// # impl CliCommand for MyCmd {
    /// #   fn help(&self) -> CliHelpScreen { CliHelpScreen::new("", "", "") }
    /// fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
    ///     req.require_params(2)?;  // Require at least 2 parameters
    ///     let source = &req.args[0];
    ///     let dest = &req.args[1];
    ///     // ... process command
    ///     Ok(())
    /// }
    /// # }
    /// ```
    pub fn require_params(&self, num: usize) -> Result<(), CliError> {
        match self.args.len() {
            len if len >= num => Ok(()),
            _ => Err(CliError::MissingParams),
        }
    }

    /// Ensures that the specified flag was provided.
    ///
    /// # Arguments
    ///
    /// * `flag` - The name of the required flag
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the flag is present, or `CliError::MissingFlag` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::{CliRequest, CliCommand, CliHelpScreen};
    /// # struct MyCmd;
    /// # impl CliCommand for MyCmd {
    /// #   fn help(&self) -> CliHelpScreen { CliHelpScreen::new("", "", "") }
    /// fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
    ///     req.require_flag("--output")?;  // Require --output flag
    ///     let output = req.get_flag("--output").unwrap();
    ///     // ... process command
    ///     Ok(())
    /// }
    /// # }
    /// ```
    pub fn require_flag(&self, flag: &str) -> Result<(), CliError> {
        if self.has_flag(&flag) {
            Ok(())
        } else {
            Err(CliError::MissingFlag(flag.to_string()))
        }
    }

    /// Gets the value of a flag if it was provided.
    ///
    /// # Arguments
    ///
    /// * `flag` - The name of the flag
    ///
    /// # Returns
    ///
    /// Returns `Some(String)` with the flag's value, or `None` if the flag wasn't provided.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::CliRequest;
    /// # fn example(req: &CliRequest) {
    /// if let Some(output) = req.get_flag("--output") {
    ///     println!("Output file: {}", output);
    /// }
    /// # }
    /// ```
    pub fn get_flag(&self, flag: &str) -> Option<String> {
        match self.flag_values.get(&flag.to_string()) {
            Some(r) => Some(r.clone()),
            None => None,
        }
    }

    /// Validates that a flag's value conforms to the specified format.
    ///
    /// # Arguments
    ///
    /// * `flag` - The name of the flag to validate
    /// * `format` - The format validator to apply
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the flag value is valid, or a `CliError` describing the issue.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::{CliRequest, CliFormat, CliCommand, CliHelpScreen};
    /// # struct MyCmd;
    /// # impl CliCommand for MyCmd {
    /// #   fn help(&self) -> CliHelpScreen { CliHelpScreen::new("", "", "") }
    /// fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
    ///     req.validate_flag("--port", CliFormat::Integer)?;
    ///     // Now we know --port contains a valid integer
    ///     Ok(())
    /// }
    /// # }
    /// ```
    pub fn validate_flag(&self, flag: &str, format: CliFormat) -> Result<(), CliError> {
        let value = self.get_flag(&flag).ok_or(CliError::MissingFlag(flag.to_string()))?;
        self.validate(0, &value, format.clone())?;
        Ok(())
    }

    /// Checks if a flag was provided.
    ///
    /// # Arguments
    ///
    /// * `flag` - The name of the flag to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the flag is present, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::CliRequest;
    /// # fn example(req: &CliRequest) {
    /// if req.has_flag("--verbose") {
    ///     println!("Verbose mode enabled");
    /// }
    /// # }
    /// ```
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(&flag.to_string()) || self.flag_values.contains_key(&flag.to_string())
    }

    /// Validates that all parameters conform to the specified formats.
    ///
    /// # Arguments
    ///
    /// * `formats` - A vector of format validators, one for each parameter position
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all parameters are valid, or a `CliError` for the first invalid parameter.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::{CliRequest, CliFormat, CliCommand, CliHelpScreen};
    /// # struct MyCmd;
    /// # impl CliCommand for MyCmd {
    /// #   fn help(&self) -> CliHelpScreen { CliHelpScreen::new("", "", "") }
    /// fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
    ///     req.validate_params(vec![
    ///         CliFormat::File,           // First arg must be a file
    ///         CliFormat::IntegerRange(1..100),  // Second arg: 1-99
    ///     ])?;
    ///     // Now we know the parameters are valid
    ///     Ok(())
    /// }
    /// # }
    /// ```
    pub fn validate_params(&self, formats: Vec<CliFormat>) -> Result<(), CliError> {
        for (pos, format) in formats.iter().enumerate() {
            let arg = self.args.get(pos).ok_or_else(|| {
                CliError::InvalidParam(pos, format!("Expected parameter at position {}", pos))
            })?;

            self.validate(pos, &arg, format.clone())?;
        }

        Ok(())
    }

    /// Validates a single value against a format specification.
    ///
    /// Internal method used by `validate_params` and `validate_flag`.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position of the parameter (for error messages)
    /// * `arg` - The value to validate
    /// * `format` - The format validator to apply
    fn validate(&self, pos: usize, arg: &str, format: CliFormat) -> Result<(), CliError> {
        match format {
            CliFormat::Any => return Ok(()),
            CliFormat::Integer => {
                arg.parse::<i64>().map_err(|_| {
                    CliError::InvalidParam(pos, format!("Expected integer, got '{}'", arg))
                })?;
            }
            CliFormat::Decimal => {
                arg.parse::<f64>().map_err(|_| {
                    CliError::InvalidParam(pos, format!("Expected decimal number, got '{}'", arg))
                })?;
            }
            CliFormat::Boolean => {
                if !["true", "false", "1", "0", "yes", "no"].contains(&arg.to_lowercase().as_str())
                {
                    return Err(CliError::InvalidParam(
                        pos,
                        format!("Expected boolean (true/false/yes/no/1/0), got '{}'", arg),
                    ));
                }
            }
            CliFormat::Email => {
                if !arg.contains('@') || !arg.contains('.') {
                    return Err(CliError::InvalidParam(
                        pos,
                        format!("Expected valid email, got '{}'", arg),
                    ));
                }
            }
            CliFormat::Url => {
                Url::parse(arg).map_err(|_| {
                    CliError::InvalidParam(pos, format!("Expected valid URL, got '{}'", arg))
                })?;
            }
            CliFormat::StringRange(range) => {
                let len = arg.len();
                if !range.contains(&len) {
                    return Err(CliError::InvalidParam(
                        pos,
                        format!(
                            "String length must be between {} and {}, got length {}",
                            range.start, range.end, len
                        ),
                    ));
                }
            }
            CliFormat::IntegerRange(range) => {
                let val = arg.parse::<i64>().map_err(|_| {
                    CliError::InvalidParam(pos, format!("Expected integer, got '{}'", arg))
                })?;
                if !range.contains(&val) {
                    return Err(CliError::InvalidParam(
                        pos,
                        format!(
                            "Integer must be between {} and {}, got {}",
                            range.start, range.end, val
                        ),
                    ));
                }
            }
            CliFormat::DecimalRange(range) => {
                let val = arg.parse::<f64>().map_err(|_| {
                    CliError::InvalidParam(pos, format!("Expected decimal, got '{}'", arg))
                })?;
                if val < range.start || val >= range.end {
                    return Err(CliError::InvalidParam(
                        pos,
                        format!(
                            "Decimal must be between {} and {}, got {}",
                            range.start, range.end, val
                        ),
                    ));
                }
            }
            CliFormat::OneOf(options) => {
                if !options.contains(&arg.to_string()) {
                    return Err(CliError::InvalidParam(
                        pos,
                        format!(
                            "Expected one of ({}), got '{}'",
                            options.join(" / ").to_string(),
                            arg
                        ),
                    ));
                }
            }
            CliFormat::File => {
                let metadata = fs::metadata(&arg)?;
                if !metadata.is_file() {
                    return Err(CliError::InvalidParam(
                        pos,
                        format!("File does not exist, '{}'", arg),
                    ));
                }
            }
            CliFormat::Directory => {
                let metadata = fs::metadata(&arg)?;
                if !metadata.is_dir() {
                    return Err(CliError::InvalidParam(
                        pos,
                        format!("Directory does not exist, '{}'", arg),
                    ));
                }
            }
        };

        Ok(())
    }
}
