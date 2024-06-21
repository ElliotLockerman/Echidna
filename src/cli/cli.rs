
use echidna_lib::config::{Config, GroupBy};
use echidna_lib::term;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    command: String,

    out_path: PathBuf,

    #[arg(long, default_value_t = String::from(""))]
    exts: String,

    #[arg(long, default_value_t = Default::default())]
    group_open_by: GroupBy,

    #[arg(long, short, default_value_t = String::from("."))]
    out_dir: String,

    #[arg(long)]
    shim_path: Option<String>,

    #[arg(long)]
    identifier: Option<String>,

    #[arg(long, short, action)]
    force: bool,

    #[arg(long, default_value_t = String::from("Terminal.app"))]
    terminal: String,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let identifier = args.identifier
        .map(|x| x.clone()) // Just to make it the same type as the default, below.
        .unwrap_or(format!("com.example.{}Opener", args.command));

    if !term::is_supported(&args.terminal) {
        eprintln!("Terminal {} is not supported", args.terminal);
    }
    let config = Config::new(args.command, args.group_open_by, args.terminal);

    let shim_path = match args.shim_path {
        Some(x) => x.into(),
        None => {
            let mut path = std::env::current_exe()
                .map_err(|e| format!("Failed to get current ext: {e}"))?;
            if !path.pop() {
                return Err(format!("Couldn't pop binary filename from path '{}' !?", path.display()));
            }
            path.push("echidna-shim");
            path
        }
    };

    if !shim_path.exists() {
        return Err(format!("Couldn't find shim executable at '{}'", shim_path.display()));
    }

    echidna_lib::generate_shim_app(
        &config,
        args.exts,
        &identifier,
        &shim_path,
        args.out_path.clone(),
        args.force,
    ).map(|_| ())
        .map_err(|e| e.to_msg(&args.out_path))
}

