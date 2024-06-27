
use echidna_lib::{term, bail, bailf};
use echidna_lib::config::{Config, GroupBy, TerminalApp};
use echidna_lib::generate::Generator;
use echidna_lib::misc::DEFAULT_UTIS;

use std::path::PathBuf;

use clap::Parser;

/// Generate a shim app.
#[derive(Parser, Debug)]
struct Args {
    /// The terminal program to execute.
    command: String,

    /// Path to new app, including app name.
    out_path: PathBuf,

    /// A comma-delimited list of Uniform Type Identifiers to support.
    #[arg(long, default_value = DEFAULT_UTIS)]
    utis: String,


    /// all: open together. none: one per window.
    #[arg(long, default_value_t = Default::default())]
    group_open_by: GroupBy,

    /// Path to the shim binary. [default: same directory as echidna-cli]
    #[arg(long)]
    shim_path: Option<String>,

    /// Bundle Identifier. [default: com.example.{command}Opener]
    #[arg(long)]
    bundle_id: Option<String>,

    /// Overwrite existing.
    #[arg(long, short, action)]
    force: bool,

    /// Terminal app to open in. Mutually exclusive with --generic-terminal.
    #[arg(
        long,
        help = String::from("Terminal app in which to open. Supported: ")
            + term::supported_terminals_string().as_str()
    )]
    terminal: Option<String>,

    /// An unsupported terminal to (attempt) to use by sending keystrokes. Mutually exclusive with
    /// --terminal.
    #[arg(long)]
    generic_terminal: Option<String>,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let bundle_id = args.bundle_id
        .unwrap_or(format!("com.example.{}Opener", args.command));

    if args.terminal.is_some() && args.generic_terminal.is_some() {
        bail!("Only one of --terminal and --generic-terminal may be passed");
    }

    let terminal = if let Some(term) = args.generic_terminal {
        TerminalApp::Generic(term.to_owned())
    } else if let Some(term) = &args.terminal {
        if !term::is_supported(term) {
            return Err(format!(
                "Terminal {} is not supported (supported terminals: {}), but you can try it with --generic-terminal", 
                term,
                term::supported_terminals_string(),
            ));
        }
        TerminalApp::Supported(term.to_owned())
    } else {
        TerminalApp::Supported("Terminal.app".to_owned())
    };

    let config = Config{
        command: args.command,
        group_open_by: args.group_open_by,
        terminal,
    };

    let shim_path = match args.shim_path {
        Some(x) => x.into(),
        None => {
            let mut path = std::env::current_exe()
                .map_err(|e| format!("Failed to get current exe: {e}"))?;
            if !path.pop() {
                bailf!("Couldn't pop binary filename from path '{}' !?", path.display());
            }
            path.push("echidna-shim");
            path
        }
    };

    if !shim_path.exists() {
        bailf!("Couldn't find shim executable at '{}'", shim_path.display());
    }

    let mut gen = Generator::gen(
        &config,
        args.utis,
        &bundle_id,
        &shim_path,
        args.out_path.clone()
    )?;

    gen.save(args.force).map(|_| ()) .map_err(|e| e.to_msg(&args.out_path))
}

