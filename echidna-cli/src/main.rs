
use echidna_util::config::{Config, GroupBy};

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    app_name: String,

    command: String,

    #[arg(long, default_value_t = String::from(""))]
    exts: String,

    #[arg(long, default_value_t = Default::default())]
    group_open_by: GroupBy,

    #[arg(long, short, default_value_t = String::from("."))]
    out_dir: String,

    #[arg(long)]
    shim_path: Option<String>,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let config = Config::new(args.command, args.group_open_by);

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
        args.app_name,
        &config,
        args.exts,
        &shim_path,
        PathBuf::from(args.out_dir)
    )
}

