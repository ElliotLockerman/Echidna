
use std::fmt;
use std::fs;
use std::path::Path;

use clap::ValueEnum;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum GroupBy {
    None,
    All,
}

impl Default for GroupBy {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum TerminalApp {
    Supported(String),
    Generic(String),
}

impl TerminalApp {
    pub fn name(&self) -> &str {
        match self {
            TerminalApp::Supported(name) => name,
            TerminalApp::Generic(name) => name,
        }
    }
}

impl fmt::Display for GroupBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // For use by clap, lower case since actual cli arguments would be lower case
        match self {
            GroupBy::None => write!(f, "none"),
            GroupBy::All => write!(f, "all"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub command: String,
    pub group_open_by: GroupBy,

    pub terminal: TerminalApp,
}

fn ts<E: ToString>(e: E) -> String {
    e.to_string()
}

impl Config {
    pub fn load() -> Result<Config, String> {
        let mut path = crate::misc::get_app_resources()?;
        path.push("config.json");

        let conf_str = std::fs::read_to_string(path).map_err(ts)?;

        let conf: Config = serde_json::from_str(&conf_str).map_err(ts)?;
        if conf.command.is_empty() {
            return Err("Config's 'command' field may not be empty".to_owned());
        }

        Ok(conf)
    }

    pub fn write(&self, resources: &Path) -> Result<(), String> {
        let config_dir = resources.join("config.json");

        let config_json = serde_json::to_string(self).map_err(|e|
            format!("Error serializing config {self:?}: {e}")
        )?;

        fs::write(&config_dir, config_json).map_err(|e|
            format!("Error writing config to temporary directory '{}': {e}", config_dir.display())
        )?;

        Ok(())
    }
}












