
use indexmap::{IndexMap, indexmap};
use crate::CliCommand;
use crate::router::CliRouter;
use crate::*;

pub struct CliHelpScreen {
    pub title: String,
    pub usage: String,
    pub description: String,
    pub params: IndexMap<String, String>,
    pub flags: IndexMap<String, String>,
    pub examples: Vec<String>
}

impl CliHelpScreen {

    pub fn new(title: &str, usage: &str, description: &str) -> Self {

        Self {
            title: title.to_string(),
            usage: usage.to_string(),
            description: description.to_string(),
            params: indexmap![],
            flags: indexmap![],
            examples: Vec::new()
        }
    }

    /// Add item to list of parameters that are displayed when the help screen is output.
    pub fn add_param(&mut self, param: &str, description: &str) {
        self.params.insert(param.to_string(), description.to_string());
    }

    /// Add item to list of flags that are displayed when the help screen is output.
    pub fn add_flag(&mut self, flag: &str, description: &str) {
        self.flags.insert(flag.to_string(), description.to_string());
    }

    /// Add item to list of examples that are displayed when the help screen is output.
    pub fn add_example(&mut self, example: &str) {
        self.examples.push(example.to_string());
    }

    // Never needs to be manually executed, and automatically executed if the first argument passed 
    /// via the command line is 'help' or '-h'.  Outputs the help screen fo the specified CLI command. 
    pub fn render(cmd: &Box<dyn CliCommand>, cmd_alias: &String, shortcuts: &Vec<String>) {

        // Get help screen
        let help = cmd.help();

        // Display basics
        cli_header(help.title.as_str());
        cli_send("USAGE\r\n\r\n");
        cli_send(format!("    {}\r\n", help.usage).as_str());

        // Display shortcuts
        for shortcut in shortcuts {
            let tmp_usage = help.usage.replace(cmd_alias, shortcut.as_str());
            cli_send(format!("    {}\r\n", tmp_usage).as_str());
        }
        cli_send("\r\n");

        // Description
        if !help.description.is_empty() {
            let options = textwrap::Options::new(75).initial_indent("    ").subsequent_indent("    ");
            let desc = textwrap::fill(help.description.as_str(), &options);

            cli_send("DESCRIPTION:\r\n\r\n");
            cli_send(desc.as_str());
            cli_send("\r\n\r\n");
        }

        // Parameters
        if !help.params.is_empty() {
            cli_send("PARAMETERS\r\n\r\n");
            cli_display_array(&help.params);
        }

        // Flags
        if !help.flags.is_empty() {
            cli_send("FLAGS\r\n\r\n");
            cli_display_array(&help.flags);
        }

        // Examples
        if !help.examples.is_empty() {
            cli_send("EXAMPLES\r\n\r\n");
            for example in help.examples {
                println!("    {}\r\n", example);
            }
        }

        // End
        cli_send("-- END --\r\n\r\n");
    }

    /// Never needs to be manually executed, and is automatically when the first and only argument passed 
    /// via command line is 'help' or '-h'.  Displays either all availalbe categories or CLI commands 
    /// depending whether or not categories have been added into the router.
    pub fn render_index(router: &CliRouter) {

        // Header
        cli_header("Available Commands");
        cli_send("Below shows all available commands.  Run any of the commands with 'help' as the first argument to view full details on the command.\r\n\r\n");
        cli_send("AVAILABLE COMMANDS\r\n\r\n");

        // Display as needed
        let mut table: IndexMap<String, String> = indexmap![];
        if !router.categories.is_empty() {

            // Sort keys
            let mut keys: Vec<String> = router.categories.keys().cloned().collect();
            keys.sort();

            // Create array to render
            for alias in keys {
                let (_title, description) = router.categories.get(&alias).unwrap();
                table.insert(alias.to_string(), description.to_string());
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
        cli_send("-- END --\r\n");
        std::process::exit(0);
    }

    /// Never needs to be manually executed, and only applicable if you're using multiple categories to organize groups 
    /// of CLI commands.  Executed first first argument via command line is either 'help' or '-h', and 
    /// second is the name of a category.  Will display all CLI commands available within that category.
    pub fn render_category(router: &CliRouter, cat_alias: &String) {

        // GEt category
        let (cat_title, cat_desc) = router.categories.get(cat_alias).unwrap();
        cli_header(cat_title);

        // Description
        if !cat_desc.is_empty() {
            let options = textwrap::Options::new(75).initial_indent("    ").subsequent_indent("    ");
            let desc = textwrap::fill(cat_desc.as_str(), &options);

            cli_send("DESCRIPTION:\r\n\r\n");
            cli_send(desc.as_str());
            cli_send("\r\n\r\n");
        }

        let chk = format!("{} ", cat_alias);
        let mut keys: Vec<String> = router.commands.keys().filter(|&k| k.starts_with(&chk)).cloned().collect();
        keys.sort();

        // GO through commands
        let mut table: IndexMap<String, String> = indexmap![];
        for alias in keys {
            let cmd = router.commands.get(&alias).unwrap();
            let cmd_help = cmd.help();
            table.insert(alias.trim_start_matches(&chk).to_string(), cmd_help.description);
        }


        // Display commands
        cli_send("AVAILABLE COMMANDS\r\n\r\n");
        cli_display_array(&table);
        cli_send("-- END --\r\n\r\n");
        std::process::exit(0);
    }

}


