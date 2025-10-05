// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use std::fmt;

/// Error types that can occur during CLI command processing.
///
/// This enum represents all possible errors that can be returned by CLI commands,
/// including missing parameters, invalid flags, and generic errors.
#[derive(Debug)]
pub enum CliError {
    /// Required parameters were not provided or are invalid.
    MissingParams,
    /// A required flag was not provided.
    MissingFlag(String),
    /// A parameter at a specific position failed validation.
    /// Contains the position (0-indexed) and an error message describing the issue.
    InvalidParam(usize, String),
    /// A generic error with a custom message.
    Generic(String),
}

impl std::error::Error for CliError {}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::MissingParams => write!(f, "Missing / invalid parameters.."),
            CliError::MissingFlag(flag) => write!(f, "Missing required flag, {}", flag),
            CliError::InvalidParam(pos, msg) => {
                write!(f, "Invalid parameter at position {}: {}", pos, msg)
            }
            CliError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::Generic(err.to_string())
    }
}
