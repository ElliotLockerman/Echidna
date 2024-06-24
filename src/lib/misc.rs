
use std::path::PathBuf;

pub const DEFAULT_UTIS: &str = "public.text";

#[macro_export]
macro_rules! bail {
    ($e:expr) => {{
        return Err($e);
    }}
}

#[macro_export]
macro_rules! bailf {
    ($($e:expr),+) => {{
        return Err(format!($($e),+));
    }}
}

// Get the path to the currently executing app bundle's Resources directory.
pub fn get_app_resources() -> Result<PathBuf, String> {
    let mut path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current exe: {e}"))?;

    // Binary itself
    if !path.pop() {
        bailf!("Couldn't pop binary filename from path '{}' !?", path.display());
    }

    // MacOS/
    if !path.pop() {
        bailf!("Couldn' MacOS/ from path '{}', is this being run in an app bundle?", path.display());
    }

    path.push("Resources");
    Ok(path)
}
