
use std::path::PathBuf;

use log::error;
use serde::Deserialize;

#[derive(Deserialize,Debug, Clone)]
pub enum GroupBy {
    None,
    All,
}

impl Default for GroupBy {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub command: String,
    pub group_open_by: GroupBy,
}

fn ts<E: ToString>(e: E) -> String {
    e.to_string()
}

impl Config {
    pub fn load() -> Result<Config, String> {
        let bin_path = match std::env::args().next() {
            Some(x) => x,
            None => {
                error!("Couldn't get binary path");
                std::process::exit(1);
            },
        };

        let mut config_path = PathBuf::from(bin_path);
        config_path.pop(); // Binary itself
        config_path.pop(); // MacOS/
        config_path.push("Resources/config.json5");

        let conf_str = std::fs::read_to_string(config_path).map_err(ts)?;

        let conf: Config = serde_json5::from_str(&conf_str).map_err(ts)?;
        if conf.command.is_empty() {
            return Err("Config's 'command' field may not be empty".to_owned());
        }

        Ok(conf)
    }
}












