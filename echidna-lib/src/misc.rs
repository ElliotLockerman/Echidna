
use std::path::PathBuf;

// Get the path to the currently executing app bundle's Resources directory.
pub fn get_app_resources() -> Result<PathBuf, String> {
    let mut path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current ext: {e}"))?;

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

