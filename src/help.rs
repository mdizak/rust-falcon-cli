// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT
use crate::CliCommand;
use crate::router::CliRouter;
use crate::*;
use indexmap::{IndexMap, indexmap};

/// Structure representing a help screen for a CLI command.
///
/// This struct contains all the information needed to render a complete help screen,
/// including title, usage, description, parameters, flags, and examples.
pub struct CliHelpScreen {
    /// The title displayed at the top of the help screen.
    pub title: String,
    /// Usage string showing how to invoke the command.
    pub usage: String,
    /// Detailed description of what the command does.
    pub description: String,
    /// Map of parameter names to their descriptions.
    pub params: IndexMap<String, String>,
    /// Map of flag names to their descriptions.
    pub flags: IndexMap<String, String>,
    /// List of example command invocations.
    pub examples: Vec<String>,
}

impl CliHelpScreen {
    /// Creates a new help screen with the specified title, usage, and description.
    ///
    /// # Arguments
    ///
    /// * `title` - The title displayed at the top of the help screen
    /// * `usage` - Usage string showing how to invoke the command
    /// * `description` - Detailed description of what the command does
    ///
    /// # Example
    ///
    /// ```
    /// use falcon_cli::CliHelpScreen;
    ///
    /// let help = CliHelpScreen::new(
    ///     "My Command",
    ///     "myapp command [OPTIONS]",
    ///     "This command does something useful"
    /// );
    /// ```
    pub fn new(title: &str, usage: &str, description: &str) -> Self {
        Self {
            title: title.to_string(),
            usage: usage.to_string(),
            description: description.to_string(),
            params: indexmap![],
            flags: indexmap![],
            examples: Vec::new(),
        }
    }

    /// Adds a parameter to the list displayed in the help screen.
    ///
    /// # Arguments
    ///
    /// * `param` - The parameter name
    /// * `description` - Description of what the parameter does
    ///
    /// # Example
    ///
    /// ```
    /// # use falcon_cli::CliHelpScreen;
    /// let mut help = CliHelpScreen::new("Title", "usage", "desc");
    /// help.add_param("filename", "The name of the file to process");
    /// ```
    pub fn add_param(&mut self, param: &str, description: &str) {
        self.params.insert(param.to_string(), description.to_string());
    }

    /// Adds a flag to the list displayed in the help screen.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag name (e.g., "--verbose" or "-v")
    /// * `description` - Description of what the flag does
    ///
    /// # Example
    ///
    /// ```
    /// # use falcon_cli::CliHelpScreen;
    /// let mut help = CliHelpScreen::new("Title", "usage", "desc");
    /// help.add_flag("--verbose|-v", "Enable verbose output");
    /// ```
    pub fn add_flag(&mut self, flag: &str, description: &str) {
        self.flags.insert(flag.to_string(), description.to_string());
    }

    /// Adds an example to the list displayed in the help screen.
    ///
    /// # Arguments
    ///
    /// * `example` - An example command invocation
    ///
    /// # Example
    ///
    /// ```
    /// # use falcon_cli::CliHelpScreen;
    /// let mut help = CliHelpScreen::new("Title", "usage", "desc");
    /// help.add_example("myapp command --verbose input.txt");
    /// ```
    pub fn add_example(&mut self, example: &str) {
        self.examples.push(example.to_string());
    }

    /// Renders and displays the help screen for a specific CLI command.
    ///
    /// This method is automatically executed when the first argument passed via the command line
    /// is 'help' or '-h'. It should not typically be called manually.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The CLI command to display help for
    /// * `cmd_alias` - The primary alias/name of the command
    /// * `shortcuts` - List of shortcut aliases for the command
    pub fn render(cmd: &Box<dyn CliCommand>, cmd_alias: &String, shortcuts: &Vec<String>) {
        // Get help screen
        let help = cmd.help();

        // Display basics
        cli_header(help.title.as_str());
        cli_sendln!("USAGE\n");
        cli_sendln!(format!("    {}\n", help.usage).as_str());

        // Display shortcuts
        for shortcut in shortcuts {
            let tmp_usage = help.usage.replace(cmd_alias, shortcut.as_str());
            cli_sendln!(format!("    {}", tmp_usage).as_str());
        }
        //cli_sendln!("");

        // Description
        if !help.description.is_empty() {
            let options =
                textwrap::Options::new(75).initial_indent("    ").subsequent_indent("    ");
            let desc = textwrap::fill(help.description.as_str(), &options);

            cli_sendln!("DESCRIPTION:\n");
            cli_sendln!(desc.as_str());
            cli_sendln!("");
        }

        // Parameters
        if !help.params.is_empty() {
            cli_sendln!("PARAMETERS\n");
            cli_display_array(&help.params);
        }

        // Flags
        if !help.flags.is_empty() {
            cli_sendln!("FLAGS\n");
            cli_display_array(&help.flags);
        }

        // Examples
        if !help.examples.is_empty() {
            cli_sendln!("EXAMPLES\n");
            for example in help.examples {
                println!("    {}\n", example);
            }
        }

        // End
        cli_sendln!("-- END --\n");
    }

    /// Renders and displays the main help index for the application.
    ///
    /// This method is automatically executed when the first and only argument passed via the
    /// command line is 'help' or '-h'. It displays either all available categories or CLI commands
    /// depending on whether categories have been added to the router.
    ///
    /// # Arguments
    ///
    /// * `router` - The CLI router containing all registered commands and categories
    pub fn render_index(router: &CliRouter) {
        // Header
        if router.app_name.is_empty() {
            cli_header("Help");
        } else {
            cli_header(&router.app_name);
        }

        // Globa flags, if we have them
        if !router.global_flags.is_empty() {
            cli_sendln!("GLOBAL FLAGS\n");
            let mut global_arr = IndexMap::new();
            for gf in router.global_flags.iter() {
                let mut key = format!("{}|{}", gf.short, gf.long);
                if gf.short.is_empty() {
                    key = gf.long.to_string();
                }
                if gf.long.is_empty() {
                    key = gf.short.to_string();
                }
                global_arr.insert(key, gf.desc.to_string());
            }
            cli_display_array(&global_arr);
        }

        cli_sendln!("AVAILABLE COMMANDS\n");
        cli_sendln!("Run any of the commands with 'help' as the first argument for details\n");

        // Display as needed
        let mut table: IndexMap<String, String> = indexmap![];
        if !router.categories.is_empty() {
            // Sort keys
            let mut keys: Vec<String> = router.categories.keys().map(|k| k.to_string()).collect();
            keys.sort();

            // Create array to render
            for cat_alias in keys {
                let cat = router.categories.get(&cat_alias).unwrap();
                table.insert(cat.alias.to_string(), cat.description.to_string());
            }

            // Render array
            cli_display_array(&table);

        // No categories, display individual commands
        } else {
            // Sort keys
            let mut keys: Vec<String> = router.commands.keys().cloned().collect();
            keys.sort();

            // Go through keys
            for alias in keys {
                let cmd = router.commands.get(&alias).unwrap();
                let cmd_help = cmd.help();

                table.insert(alias.to_string(), cmd_help.description);
            }

            // Display commands
            cli_display_array(&table);
        }

        // Exit
        cli_sendln!("-- END --\r\n");
        exit(0);
    }

    /// Renders and displays help for a specific category.
    ///
    /// This method is only applicable when using multiple categories to organize groups of CLI commands.
    /// It is automatically executed when the first argument via command line is either 'help' or '-h',
    /// and the second argument is the name of a category. It displays all CLI commands available within that category.
    ///
    /// # Arguments
    ///
    /// * `router` - The CLI router containing all registered commands and categories
    /// * `cat_alias` - The alias/name of the category to display
    pub fn render_category(router: &CliRouter, cat_alias: &String) {
        // GEt category
        let cat = router.categories.get(&cat_alias.to_string()).unwrap();
        cli_header(&cat.title);

        // Description
        if !cat.description.is_empty() {
            let options =
                textwrap::Options::new(75).initial_indent("    ").subsequent_indent("    ");
            let desc = textwrap::fill(cat.description.as_str(), &options);

            cli_sendln!("DESCRIPTION:\n");
            cli_sendln!(desc.as_str());
        }

        // Sub categories
        let chk = format!("{} ", cat_alias);
        let mut sub_categories: Vec<String> =
            router.categories.keys().filter(|&k| k.starts_with(&chk)).cloned().collect();
        sub_categories.sort();

        // Go through sub-categories
        let mut table: IndexMap<String, String> = indexmap![];
        for full_alias in sub_categories {
            let alias = full_alias.trim_start_matches(&chk).to_string();
            if alias.contains(" ") {
                continue;
            }
            let desc = router.categories.get(&full_alias).unwrap().description.to_string();

            table.insert(alias, desc.clone());
        }

        // Get commands to display
        let mut keys: Vec<String> =
            router.commands.keys().filter(|&k| k.starts_with(&chk)).cloned().collect();
        keys.sort();

        // GO through commands
        for full_alias in keys {
            let alias = full_alias.trim_start_matches(&chk).to_string();
            if alias.contains(" ") {
                continue;
            }
            let cmd = router.commands.get(&full_alias).unwrap();
            let cmd_help = cmd.help();
            table.insert(alias, cmd_help.description);
        }

        // Display commands
        cli_sendln!("AVAILABLE COMMANDS\n");
        cli_display_array(&table);
        cli_sendln!("-- END --\n");
        std::process::exit(0);
    }
}
