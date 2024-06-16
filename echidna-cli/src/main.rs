
use echidna_helpers::config::{Config, GroupBy};

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
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let config = Config::new(args.command, args.group_open_by);

    echidna_lib::generate_shim_app(
        args.app_name,
        &config,
        args.exts,
        PathBuf::from(args.out_dir)
    )
}

