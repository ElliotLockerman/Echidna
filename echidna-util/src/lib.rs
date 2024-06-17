
pub mod config;

use std::path::PathBuf;

// Get the path to the currently executing app bundle's Resources directory.
pub fn get_app_resources() -> Result<PathBuf, String> {
    let bin_path = match std::env::args().next() {
        Some(x) => x,
        None => {
            return Err("Couldn't get binary path".to_owned());
        },
    };

    let mut path = PathBuf::from(bin_path);

    // Binary itself
    if !path.pop() {
        return Err(format!("Couldn't pop binary filename from path '{}' !?", path.display()));
    }

    // MacOS/
    if !path.pop() {
        return Err(format!("Couldn' MacOS/ from path '{}', is this being run in an app bundle?", path.display()));
    }

    path.push("Resources");
    Ok(path)
}

