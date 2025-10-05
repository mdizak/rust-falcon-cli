// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::{CliCommand, CliHelpScreen, CliRequest};
use crate::*;
use std::collections::HashMap;
use std::env;
use strsim::levenshtein;

/// The main router for CLI commands.
///
/// This struct manages all registered commands, categories, and global flags.
/// It handles parsing command line arguments and routing them to the appropriate
/// command handler.
#[derive(Default)]
pub struct CliRouter {
    /// The application name displayed in help screens.
    pub app_name: String,
    /// Version message displayed with -v or --version flags.
    pub version_message: String,
    /// Internal: Alias of the handler for this router node.
    pub handler_alias: Option<String>,
    /// Map of command aliases to their handlers.
    pub handlers: HashMap<String, CliHandler>,
    /// Map of command aliases to their implementations.
    pub commands: HashMap<String, Box<dyn CliCommand>>,
    /// Map of category aliases to their definitions.
    pub categories: HashMap<String, CliCategory>,
    /// Flags to ignore during command lookup.
    pub ignore_flags: HashMap<String, bool>,
    /// List of global flags available to all commands.
    pub global_flags: Vec<CliGlobalFlag>,
    /// Internal: Whether global flags have been parsed.
    pub parsed_global_flags: bool,
    /// Internal: Child routers for nested command structures.
    pub children: HashMap<String, Box<CliRouter>>,
}

/// Handler configuration for a CLI command.
///
/// Contains metadata about how a command should be invoked and parsed.
#[derive(Clone)]
pub struct CliHandler {
    /// The primary alias for the command.
    pub alias: String,
    /// Alternate shortcuts for invoking the command.
    pub shortcuts: Vec<String>,
    /// Flags that expect a value (e.g., `--output filename`).
    pub value_flags: Vec<String>,
}

/// A category for organizing related commands.
///
/// Categories are displayed in the help index and can contain multiple commands.
#[derive(Clone)]
pub struct CliCategory {
    /// The category's alias/identifier.
    pub alias: String,
    /// The display title for the category.
    pub title: String,
    /// A description of what commands in this category do.
    pub description: String,
}

/// A global flag available to all commands.
///
/// Global flags are processed before command routing and can be accessed
/// via the router's `has_global()` and `get_global()` methods.
#[derive(Clone, Default)]
pub struct CliGlobalFlag {
    /// Short form of the flag (e.g., "-v").
    pub short: String,
    /// Long form of the flag (e.g., "--verbose").
    pub long: String,
    /// Description of what the flag does.
    pub desc: String,
    /// Whether this flag expects a value.
    pub is_value: bool,
    /// Whether this flag was provided.
    pub has: bool,
    /// The value provided with this flag (if applicable).
    pub value: Option<String>,
}

impl CliRouter {
    /// Creates a new CLI router.
    ///
    /// # Example
    ///
    /// ```
    /// use falcon_cli::CliRouter;
    ///
    /// let mut router = CliRouter::new();
    /// router.app_name("My Application");
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a command with the router.
    ///
    /// Links a struct that implements `CliCommand` to a command name, along with
    /// optional shortcuts and flags that expect values.
    ///
    /// # Arguments
    ///
    /// * `alias` - The full name of the command
    /// * `shortcuts` - Vector of alternate ways to invoke the command
    /// * `value_flags` - Vector of flags that expect a value (e.g., `["--output", "--config"]`)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::{CliRouter, CliCommand, CliRequest, CliHelpScreen};
    /// # #[derive(Default)]
    /// # struct BuildCommand;
    /// # impl CliCommand for BuildCommand {
    /// #   fn process(&self, req: &CliRequest) -> anyhow::Result<()> { Ok(()) }
    /// #   fn help(&self) -> CliHelpScreen { CliHelpScreen::new("", "", "") }
    /// # }
    /// let mut router = CliRouter::new();
    /// router.add::<BuildCommand>(
    ///     "build",
    ///     vec!["b"],
    ///     vec!["--output", "--config"]
    /// );
    /// ```
    pub fn add<T>(&mut self, alias: &str, shortcuts: Vec<&str>, value_flags: Vec<&str>)
    where
        T: CliCommand + Default + 'static,
    {
        // Set handler
        let handler = CliHandler {
            alias: alias.to_lowercase(),
            shortcuts: shortcuts.clone().into_iter().map(|s| s.to_string()).collect(),
            value_flags: value_flags.clone().into_iter().map(|s| s.to_string()).collect(),
        };
        self.handlers.insert(alias.to_string(), handler.clone());
        self.commands.insert(alias.to_lowercase(), Box::<T>::default());

        // Set queue to  add
        let mut queue: Vec<String> = shortcuts.clone().into_iter().map(|s| s.to_string()).collect();
        queue.insert(0, alias.to_string());

        // Add queue
        for cmd_alias in queue.iter() {
            let mut child = &mut *self;
            for segment in cmd_alias.split_whitespace() {
                child =
                    child.children.entry(segment.to_string()).or_insert(Box::new(CliRouter::new()));
            }
            child.handler_alias = Some(handler.alias.to_string());
        }
    }

    /// Sets the application name displayed in help screens.
    ///
    /// # Arguments
    ///
    /// * `name` - The application name
    ///
    /// # Example
    ///
    /// ```
    /// # use falcon_cli::CliRouter;
    /// let mut router = CliRouter::new();
    /// router.app_name("MyApp v1.0");
    /// ```
    pub fn app_name(&mut self, name: &str) {
        self.app_name = name.to_string();
    }

    /// Sets the version message displayed with -v or --version.
    ///
    /// # Arguments
    ///
    /// * `msg` - The version message
    ///
    /// # Example
    ///
    /// ```
    /// # use falcon_cli::CliRouter;
    /// let mut router = CliRouter::new();
    /// router.version_message("MyApp version 1.0.0");
    /// ```
    pub fn version_message(&mut self, msg: &str) {
        self.version_message = msg.to_string();
    }

    /// Registers a global flag available to all commands.
    ///
    /// Global flags are processed before command routing and can be checked
    /// using `has_global()` or retrieved using `get_global()`.
    ///
    /// # Arguments
    ///
    /// * `short` - Short form of the flag (e.g., "-v")
    /// * `long` - Long form of the flag (e.g., "--verbose")
    /// * `is_value` - Whether the flag expects a value
    /// * `desc` - Description of what the flag does
    ///
    /// # Example
    ///
    /// ```
    /// # use falcon_cli::CliRouter;
    /// let mut router = CliRouter::new();
    /// router.global("-v", "--verbose", false, "Enable verbose output");
    /// router.global("-c", "--config", true, "Specify config file");
    /// ```
    pub fn global(&mut self, short: &str, long: &str, is_value: bool, desc: &str) {
        self.global_flags.push(CliGlobalFlag {
            short: short.to_string(),
            long: long.to_string(),
            is_value,
            desc: desc.to_string(),
            ..Default::default()
        });
    }

    /// Checks if a global flag was provided.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag to check (short or long form)
    ///
    /// # Returns
    ///
    /// Returns `true` if the flag was provided, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::CliRouter;
    /// let mut router = CliRouter::new();
    /// router.global("-v", "--verbose", false, "Verbose output");
    /// if router.has_global("-v") {
    ///     println!("Verbose mode enabled");
    /// }
    /// ```
    pub fn has_global(&mut self, flag: &str) -> bool {
        if !self.parsed_global_flags {
            self.get_raw_args();
        }
        let flag_chk = flag.to_string();

        if let Some(index) =
            self.global_flags.iter().position(|gf| gf.short == flag_chk || gf.long == flag_chk)
        {
            return self.global_flags[index].has;
        }

        false
    }

    /// Gets the value of a global flag.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag to retrieve (short or long form)
    ///
    /// # Returns
    ///
    /// Returns `Some(String)` with the flag's value, or `None` if not provided or not a value flag.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use falcon_cli::CliRouter;
    /// let mut router = CliRouter::new();
    /// router.global("-c", "--config", true, "Config file");
    /// if let Some(config) = router.get_global("--config") {
    ///     println!("Using config: {}", config);
    /// }
    /// ```
    pub fn get_global(&mut self, flag: &str) -> Option<String> {
        if !self.parsed_global_flags {
            self.get_raw_args();
        }
        let flag_chk = flag.to_string();

        if let Some(index) =
            self.global_flags.iter().position(|gf| gf.short == flag_chk || gf.long == flag_chk)
        {
            return self.global_flags[index].value.clone();
        }

        None
    }

    /// Adds a flag to ignore during command lookup.
    ///
    /// Ignored flags are stripped from arguments before command routing occurs.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag to ignore
    /// * `is_value` - Whether the flag expects a value (which should also be ignored)
    ///
    /// # Example
    ///
    /// ```
    /// # use falcon_cli::CliRouter;
    /// let mut router = CliRouter::new();
    /// router.ignore("--internal-flag", false);
    /// router.ignore("--debug-port", true);
    /// ```
    pub fn ignore(&mut self, flag: &str, is_value: bool) {
        self.ignore_flags.insert(flag.to_string(), is_value);
    }

    /// Looks up and routes to the appropriate command handler.
    ///
    /// This method parses command line arguments, determines which command to execute,
    /// and returns the parsed request along with the command handler. It is automatically
    /// called by `cli_run()` and typically should not be called manually.
    ///
    /// # Returns
    ///
    /// Returns `Some((CliRequest, &Box<dyn CliCommand>))` if a command was found,
    /// or `None` if no command matched.
    pub fn lookup(&mut self) -> Option<(CliRequest, &Box<dyn CliCommand>)> {
        // Get raw args from command line, after filtering ignore flags out
        let mut args = self.get_raw_args()?;

        // Check for help

        let is_help = self.is_help(&mut args);
        // Lookup handler
        let handler = self.lookup_handler(&mut args)?;

        // Gather flags
        let (flags, flag_values) = self.gather_flags(&mut args, &handler);

        // Return
        let req = CliRequest {
            cmd_alias: handler.alias.to_string(),
            is_help,
            args,
            flags,
            flag_values,
            shortcuts: handler.shortcuts.to_vec(),
        };

        let cmd = self.commands.get(&handler.alias).unwrap();
        Some((req, cmd))
    }

    fn get_raw_args(&mut self) -> Option<Vec<String>> {
        let mut cmd_args = vec![];
        let mut skip_next = true;
        let mut global_value_index: Option<usize> = None;
        self.parsed_global_flags = true;

        for value in env::args() {
            if skip_next {
                skip_next = false;
                if let Some(index) = global_value_index {
                    self.global_flags[index].value = Some(value.to_string());
                    global_value_index = None;
                }
                continue;
            }

            if ["-v", "--version"].contains(&value.as_str()) && !self.version_message.is_empty() {
                println!("{}", self.version_message);
                std::process::exit(0);
            } else if let Some(is_value) = self.ignore_flags.get(&value) {
                skip_next = *is_value;
            } else if let Some(index) = self
                .global_flags
                .iter()
                .position(|gf| [gf.short.to_string(), gf.long.to_string()].contains(&value))
            {
                skip_next = self.global_flags[index].is_value;
                if skip_next {
                    global_value_index = Some(index);
                }
            } else {
                cmd_args.push(value.to_string());
            }
        }

        if !cmd_args.is_empty() {
            Some(cmd_args)
        } else {
            None
        }
    }

    /// Check for help being requested
    fn is_help(&self, args: &mut Vec<String>) -> bool {
        let mut is_help = false;
        if ["help", "-h"].contains(&args[0].as_str()) {
            is_help = true;
            args.remove(0);

            if args.is_empty() {
                CliHelpScreen::render_index(self);
            }

            // Check category help
            let cat_alias = args.join(" ").to_string();
            if self.categories.contains_key(&cat_alias) {
                CliHelpScreen::render_category(&self, &cat_alias);
            }
        }

        is_help
    }

    fn lookup_handler(&self, args: &mut Vec<String>) -> Option<CliHandler> {
        let mut h_alias: Option<String> = None;
        let (mut start, mut length) = (0, 0);

        let mut child = self;
        for (pos, segment) in args.iter().enumerate() {
            if segment.starts_with("-") {
                continue;
            }

            if let Some(next) = child.children.get(&segment.to_lowercase()) {
                if length == 0 {
                    (start, length) = (pos, 1);
                } else {
                    length += 1;
                }

                if let Some(h_child) = &next.handler_alias {
                    h_alias = Some(h_child.clone());
                }
                child = next;
            } else if h_alias.is_some() {
                break;
            } else {
                child = self;
                length = 0;
            }
        }

        // Check for typos, if none
        if h_alias.is_none() {
            h_alias = self.lookup_similar(args);
        } else if h_alias.is_some() {
            args.drain(start..start + length);
        } else {
            return None;
        }

        let handler = self.handlers.get(&h_alias?)?;
        Some(handler.clone())
    }

    fn gather_flags(
        &self,
        args: &mut Vec<String>,
        handler: &CliHandler,
    ) -> (Vec<String>, HashMap<String, String>) {
        let mut incl_value = false;
        let mut flags = vec![];
        let mut flag_values: HashMap<String, String> = HashMap::new();
        let mut final_args = vec![];

        // Iterate over args
        for (pos, value) in args.iter().enumerate() {
            if incl_value {
                flag_values.insert(args[pos - 1].to_string(), value.to_string());
                incl_value = false;
            } else if value.starts_with("-") && handler.value_flags.contains(&value) {
                incl_value = true;
            } else if value.starts_with("--") {
                flags.push(value.to_string());
            } else if value.starts_with("-") {
                for char in value[1..].chars() {
                    flags.push(format!("-{}", char));
                }
            } else {
                final_args.push(value.to_string());
            }
        }

        *args = final_args;
        (flags, flag_values)
    }

    /// Attempts to find a similar command when an exact match isn't found.
    ///
    /// Uses Levenshtein distance to find commands that closely resemble the input,
    /// handling potential typos. If a close match is found, prompts the user for confirmation.
    /// This method is called automatically by `lookup()`.
    ///
    /// # Arguments
    ///
    /// * `args` - The command line arguments to search against
    ///
    /// # Returns
    ///
    /// Returns `Some(String)` with the corrected command name if found and confirmed,
    /// or `None` otherwise.
    fn lookup_similar(&self, args: &mut Vec<String>) -> Option<String> {
        let start = args.iter().position(|a| !a.starts_with("-")).unwrap_or(0);
        let search_args =
            args.clone().into_iter().filter(|a| !a.starts_with("-")).collect::<Vec<String>>();

        // Get available commands to search
        let mut commands: Vec<String> = self.commands.keys().map(|c| c.to_string()).collect();
        commands.sort_by(|a, b| {
            let a_count = a.chars().filter(|c| c.is_whitespace()).count();
            let b_count = b.chars().filter(|c| c.is_whitespace()).count();
            b_count.cmp(&a_count)
        });
        let (mut distance, mut bin_length, mut found_cmd) = (0, 0, String::new());

        // Go through commands
        for chk_alias in commands {
            let length = chk_alias.chars().filter(|c| c.is_whitespace()).count() + 1;

            // Check lowest distance, if we're completed a bin
            if bin_length != length && bin_length > 0 && distance > 0 && distance < 4 {
                let confirm_msg = format!(
                    "No command with that name exists, but a similar command with the name '{}' does exist.  Is this the command you wish to run?",
                    found_cmd
                );
                if cli_confirm(&confirm_msg) {
                    let end = (start + length).min(args.len());
                    args.drain(start..end);
                    return Some(found_cmd);
                } else {
                    return None;
                }
            } else if bin_length != length {
                bin_length = length;
                distance = 0;
                found_cmd = String::new();
            }

            let end = search_args.len().min(length);
            let search_str = search_args[..end].join(" ").to_string();

            let chk_distance = levenshtein(&chk_alias, &search_str);
            if chk_distance < distance || distance == 0 {
                distance = chk_distance;
                found_cmd = chk_alias.to_string();
            }
        }

        None
    }

    /// Adds a category for organizing related commands.
    ///
    /// Categories are displayed in the help index and can contain multiple commands.
    /// Useful for organizing large CLI applications with many commands.
    ///
    /// # Arguments
    ///
    /// * `alias` - The category's identifier
    /// * `title` - The display title for the category
    /// * `description` - Description of what commands in this category do
    ///
    /// # Example
    ///
    /// ```
    /// # use falcon_cli::CliRouter;
    /// let mut router = CliRouter::new();
    /// router.add_category("database", "Database Commands", "Manage database operations");
    /// router.add_category("user", "User Commands", "Manage user accounts");
    /// ```
    pub fn add_category(&mut self, alias: &str, title: &str, description: &str) {
        self.categories.insert(
            alias.to_lowercase(),
            CliCategory {
                alias: alias.to_lowercase(),
                title: title.to_string(),
                description: description.to_string(),
            },
        );
    }
}
