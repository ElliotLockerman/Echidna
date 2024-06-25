
use echidna_lib::config::{Config, GroupBy};
use echidna_lib::term;
use echidna_lib::bail;

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::os::unix::ffi::OsStringExt;

use core::str::FromStr;

use cacao::appkit::{App, AppDelegate, Alert};
use url::Url;
use log::{error, info};
use std::env::VarError;
use shell_quote::Bash;


fn init_log() {
    const LEVEL_KEY: &str = "ECH_SHIM_LOG_LEVEL";
    const LEVEL_PATH: &str = "ECH_SHIM_LOG_PATH";

    // Default filename if path given is a directory, default directory is $HOME, or no $HOME, /
    const DEFAULT_FILE: &str = "ech_shim_log.txt";

    let mut path = match std::env::var(LEVEL_PATH) {
        Ok(x) => PathBuf::from(x),
        Err(VarError::NotPresent) => PathBuf::new(),
        Err(VarError::NotUnicode(x)) => PathBuf::from(x),
    };

    // Maintaining the convention that the path should always be set to _something_
    // as close to valid as can be managed, and the level controlls whether
    // logging should occur.
    if path == PathBuf::new() {
        path = home::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    }

    if path.is_dir() {
        path.push(DEFAULT_FILE);
    }

    let level = match std::env::var(LEVEL_KEY) {
        Ok(x) => {
            Some(log::Level::from_str(x.as_str()).map_err(|e| e.to_string())).transpose()
        },
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(x)) => {
            Err(format!("Non-unicode {LEVEL_KEY} value: {}", x.to_string_lossy()))
        }
    };

    match level {
        Ok(Some(lev)) => { 
            simple_logging::log_to_file(path, lev.to_level_filter()).unwrap(); 
        },
        Ok(None) => (), // No logging requested
        Err(msg) => { 
            simple_logging::log_to_file(path, log::Level::max().to_level_filter()).unwrap(); 
            error!("Invalid filter {msg}, set to max");
        }
    }
}

fn modal<T: AsRef<str>, M: AsRef<str>>(title: T, msg: M) {
    let (title, msg) = (title.as_ref(), msg.as_ref());
    error!("modal {title}: {msg}");
    Alert::new(title, msg).show();
}

// Extend an OsString with anything that implements AsRef<OsStr>.
macro_rules! os_extend {
    ($os_string:ident, $($vals:expr),*) => {{
        $os_string.extend([$($vals.as_ref()),+]);
    }};
}


// Concatenate anything that implements AsRef<OsStr> into an OsString.
macro_rules! os_cat {
    ($($vals:expr),*) => {{
        let mut os_string = OsString::new();
        os_extend!(os_string, $($vals),+);
        os_string
    }};
}

fn bash_quote<S: AsRef<OsStr>>(string: S) -> OsString {
    let string = string.as_ref();
    OsString::from_vec(Bash::quote(string))
}

////////////////////////////////////////////////////////////////////////////////

struct EchidnaShimDelegate {
    config: Config,
}

impl EchidnaShimDelegate {
    fn new(config: Config) -> Self {
        Self{config}
    }

    fn run_term(&self, bash: &OsStr) {
        if let Err(e) = term::run_in_new_window(&self.config, bash) {
            modal("Error", format!("Error running `{bash:?}`: {e}"));
        }
    }
}

impl AppDelegate for EchidnaShimDelegate {

    fn open_urls(&self, urls: Vec<Url>) {
        info!("Got urls {urls:?}");

        let paths: Vec<_> = urls.into_iter().filter_map(|url| {
            if url.scheme() != "file" {
                modal("Error", format!("Only 'file' schemes are supported, '{url}''s scheme is {}", url.scheme()));
                return None;
            }

            let path: PathBuf = match url.to_file_path() {
                Ok(x) => x,
                Err(_) => {
                    modal("Error", format!("'{url}' has no path"));
                    return None;
                },
            };
            Some(path)
        }).collect();

        if paths.is_empty() {
            std::process::exit(0);
        }

        let mut cmd = OsString::new();
        if let Some(parent) = paths[0].parent() {
            let parent = bash_quote(parent);
            cmd = os_cat!("cd ", &parent, "; ");
        }

        match self.config.group_open_by {
            GroupBy::All => {
                // os_extend!(cmd, &self.config.command);
                cmd.push(&self.config.command);
                for path in paths {
                    let path = bash_quote(path);
                    os_extend!(cmd, " ", &path);
                }
                self.run_term(&cmd);
            },
            GroupBy::None => {
                for path in paths {
                    let path = bash_quote(path);
                    let cmd2 = os_cat!(&cmd, &self.config.command, " ", &path);
                    self.run_term(&cmd2);
                }
            }
        }

        // In Swift I would quit by getting a reference to the shared NSApplication,
        // but I don't see a way to do it with cacao. This doesn't seem to do
        // any harm.
        std::process::exit(0);
    }
}



fn main() -> Result<(), String> {
    init_log();

    let config = match Config::load() {
        Ok(x) => x,
        Err(msg) => {
            modal("Error loading config", &msg);
            bail!(msg);
        },
    };

    App::new("com.lockerman.EchidnaShim", EchidnaShimDelegate::new(config)).run();

    Ok(())
}
