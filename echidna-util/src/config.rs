
use serde::{Serialize, Deserialize};
use clap::ValueEnum;

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

impl std::string::ToString for GroupBy {
    fn to_string(&self) -> String {
        // For use by clap, lower case since actual cli arguments would be lower case
        match self {
            GroupBy::None => "none".to_owned(),
            GroupBy::All => "all".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub command: String,
    pub group_open_by: GroupBy,
}

fn ts<E: ToString>(e: E) -> String {
    e.to_string()
}

impl Config {
    pub fn new(command: String, group_open_by: GroupBy) -> Config {
        Config{command, group_open_by}
    }

    pub fn load() -> Result<Config, String> {
        let mut path = crate::get_app_resources()?;
        path.push("config.json5");

        let conf_str = std::fs::read_to_string(path).map_err(ts)?;

        let conf: Config = serde_json5::from_str(&conf_str).map_err(ts)?;
        if conf.command.is_empty() {
            return Err("Config's 'command' field may not be empty".to_owned());
        }

        Ok(conf)
    }
}












