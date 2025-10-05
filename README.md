
![GitHub release (latest)](https://img.shields.io/github/v/release/mdizak/rust-falcon-cli)
![License](https://img.shields.io/github/license/mdizak/rust-falcon-cli)
![Downloads](https://img.shields.io/github/downloads/mdizak/rust-falcon-cli/total)


# Falcon CLI

Efficiently develop fully featured CLI applications in Rust with minimal boilerplate.

## Features

* Command routing with automatic help screens
* Built-in input functions (text, password, confirmation, multi-select)
* Table and formatted output displays
* Parameter and flag validation
* Progress bars and text editor integration
* Smart typo correction using Levenshtein distance
* Multi-level command categories
* Global flags support

## Installation

```toml
[dependencies]
falcon-cli = "0.2"
```

## Usage

### Define a Command

```rust
use falcon_cli::*;

#[derive(Default)]
pub struct CreateDomain {}

impl CliCommand for CreateDomain {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        req.require_params(1)?;
        req.require_flag("--ip-address")?;

        let domain = &req.args[0];
        let ip = req.get_flag("--ip-address").unwrap();

        if !req.has_flag("-n") && !cli_confirm(&format!("Create {} at {}?", domain, ip)) {
            return Ok(());
        }

        // Your logic here

        cli_sendln!("Domain created: {}", domain);
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Create Domain",
            "mycli domain create <DOMAIN> --ip-address <IP> [-n]",
            "Create a new domain name"
        );
        help.add_param("DOMAIN", "Domain name to create");
        help.add_flag("--ip-address", "IP address for the domain");
        help.add_flag("-n", "Skip confirmation");
        help
    }
}
```

### Set Up Router

```rust
use falcon_cli::*;

fn main() {
    let mut router = CliRouter::new();
    router.app_name("MyApp v1.0");
    router.add::<CreateDomain>("domain create", vec!["dom c"], vec!["--ip-address"]);
    cli_run(&mut router);
}
```

Run with: `cargo run -- domain create example.com --ip-address 1.2.3.4`

### Categories

Organize commands into groups:

```rust
router.add_category("domain", "Domain Names", "Manage domains");
router.add::<CreateDomain>("domain create", vec![], vec!["--ip-address"]);
router.add::<ListDomains>("domain list", vec![], vec![]);
```

### Validation

```rust
req.validate_params(vec![
    CliFormat::Email,
    CliFormat::IntegerRange(1..100),
])?;
req.validate_flag("--port", CliFormat::Integer)?;
```

Available validators: `Any`, `Integer`, `Decimal`, `Boolean`, `Email`, `Url`, `File`, `Directory`, `IntegerRange`, `DecimalRange`, `StringRange`, `OneOf`

### User Input

```rust
let name = cli_get_input("Name: ", "default");
let password = cli_get_password("Password: ", false);
let strong_pwd = cli_get_new_password(3);  // Strength 0-4

if cli_confirm("Continue?") { /* ... */ }

let choice = cli_get_option("Pick one:", &indexmap! {
    1 => "Option A",
    2 => "Option B",
});
```

### Display

```rust
cli_header("My App");

cli_display_table(&["Name", "Age"], &vec![
    vec!["Alice", "30"],
    vec!["Bob", "25"],
]);

let mut bar = cli_progress_bar("Loading", 100);
bar.increment(10);
bar.finish();

cli_sendln!("Hello {}", name);  // Word-wrapped output
```

### Global Flags

```rust
router.global("-v", "--verbose", false, "Verbose output");
router.global("-c", "--config", true, "Config file");

if router.has_global("--verbose") { /* ... */ }
if let Some(cfg) = router.get_global("--config") { /* ... */ }
```

## Related Project

If you found this software helpful, check out [Cicero](https://cicero.sh/latest) - a self-hosted AI assistant focused on protecting personal privacy in the age of AI.

[https://cicero.sh/latest](https://cicero.sh/latest)

