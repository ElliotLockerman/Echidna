
use echidna_lib::config::{Config, GroupBy};

use std::process::Command;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use core::str::FromStr;

use cacao::appkit::{App, AppDelegate, Alert};
use url::Url;
use log::{error, info};
use std::env::VarError;

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

////////////////////////////////////////////////////////////////////////////////

fn modal<T: AsRef<str>, M: AsRef<str>>(title: T, msg: M) {
    let (title, msg) = (title.as_ref(), msg.as_ref());
    error!("modal {title}: {msg}");
    Alert::new(title, msg).show()
}

// I don't care about the extra allocations anymore, I'm just so sick of dealing
// with OsStr/OsString
macro_rules! os_cat {
    ($only:expr) => {
        OsString::from($only)
    };
    ($first:expr, $($rest:expr),+) => {{
        let mut os_string = OsString::from($first); 
        os_string.push(os_cat!($($rest),+));
        os_string
    }};
}

////////////////////////////////////////////////////////////////////////////////

struct EchidnaShimDelegate {
    config: Config,
}

impl EchidnaShimDelegate {
    fn new(config: Config) -> Self {
        Self{config}
    }
}

impl AppDelegate for EchidnaShimDelegate {

    fn open_urls(&self, urls: Vec<Url>) {
        info!("Got urls {urls:?}");

        let paths: Vec<_> = urls.into_iter().filter_map(|url| {
            if url.scheme() != "file" {
                modal("Error", format!("'only 'file' schemes are supported, '{url}''s scheme is {}", url.scheme()));
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
            cmd = os_cat!("cd ", "'", parent, "'; ");
        }

        match self.config.group_open_by {
            GroupBy::All => {
                cmd = os_cat!(cmd, &self.config.command);
                for path in paths {
                    cmd = os_cat!(cmd, " '", path, "'");
                }
                run_script_or_modal(cmd);
            },
            GroupBy::None => {
                for path in paths {
                    let cmd2 = os_cat!(cmd.clone(), &self.config.command, " '", path, "'");
                    run_script_or_modal(cmd2);
                }
            }
        }

        // In Swift I would quit by getting a reference to the shared NSApplication,
        // but I don't see a way to do it with cacao. This doesn't seem to do
        // any harm.
        std::process::exit(0);
    }
}

fn run_script_or_modal<S: AsRef<OsStr>>(bash: S) {
    let bash = bash.as_ref();
    if let Err(e) = run_script(bash) {
        modal("Error", format!("Error running `{bash:?}`: {e}"));
    }
}

fn run_script<S: AsRef<OsStr>>(bash: S) -> Result<(), String> {
    // TODO: figure out escaping edge cases.
    let js = os_cat!(r#"Application("Terminal").doScript(""#, bash.as_ref(), r#"");"#);
    run(
        "osascript",
        [
            "-l".into(), "JavaScript".into(),
            "-e".into(), js,
        ]
    )
}

fn run<C, A, S>(cmd: C, args: A) -> Result<(), String> 
    where 
        C: AsRef<OsStr>,
        A: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr> {
    let output = Command::new(cmd.as_ref())
        .args(args)
        .output()
        .map_err(|e| "Run error: ".to_owned() + e.to_string().as_str() + "\n")?;

    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Command \"{msg}\" exited with with an error: {msg}\n"));
    }

    Ok(())
}

fn main() -> Result<(), String> {
    init_log();

    let config = match Config::load() {
        Ok(x) => x,
        Err(msg) => {
            modal("Error loading config", &msg);
            return Err(msg);
        },
    };

    App::new("com.lockerman.EchidnaShim", EchidnaShimDelegate::new(config)).run();

    Ok(())
}
