
use crate::bailf;

use std::process::{Command, Stdio};
use std::io::Write;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use lazy_static::lazy_static;
use indexmap::{IndexMap,indexmap};

type RunInNewWindow = fn(bash: &OsStr) -> Result<(), String>;

lazy_static! {
    // TERMINALS and TERM_ORDER must have exactly the same keys!!!
    static ref TERMINALS: IndexMap<String, RunInNewWindow> = indexmap! {
        "Terminal.app".into() => terminal_dot_app::run_in_new_window as RunInNewWindow,
        "iTerm2".into() => iterm::run_in_new_window as RunInNewWindow,
    };
}


pub fn run_in_new_window(terminal: &str, bash: &OsStr) -> Result<(), String> {
    match TERMINALS.get(terminal) {
        Some(t) => t(bash),
        None => Err(format!("Terminal '{terminal}' is not supported")),
    }
}

pub fn default_terminal() -> &'static str {
    TERMINALS.keys().next().unwrap().as_str()
}

pub fn supported_terminals() -> impl IntoIterator<Item = &'static str> {
    TERMINALS.keys().map(|x| x.as_str())
}

pub fn supported_terminals_string() -> String {
    itertools::join(supported_terminals(), ", ")
}

pub fn is_supported(terminal: &str) -> bool {
    TERMINALS.contains_key(terminal)
}

////////////////////////////////////////////////////////////////////////////////

type JxaResult = Result<(), String>;

fn run_jxa(jxa: &OsStr, arg: &OsStr) -> JxaResult {
    let cmd = "osascript";
    let args = [OsStr::new("-lJavaScript"), OsStr::new("-"), arg];

    let mut child = Command::new::<&OsStr>(cmd.as_ref())
        .args(args)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| "Run error: ".to_owned() + e.to_string().as_str() + "\n")?;

    /* scope to close stdin and unblock osascript */ {
        let mut child_stdin = child.stdin.take().ok_or("Couldn't get child's stdin".to_owned())?;
        child_stdin.write(jxa.as_bytes()).map_err(|e| format!("Couldn't write to child's stdin: {e}"))?;
    }

    let output = child.wait_with_output().map_err(|e| format!("Error waiting on child: {e}"))?;
    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr);
        bailf!("Command '{msg}' exited with with an error: {msg}\n");
    }

    Ok(())
}


////////////////////////////////////////////////////////////////////////////////


// MacOS's built-in terminal
mod terminal_dot_app {
    use std::ffi::OsStr;

    const JXA_RUN: &str = r#"
        function run(argv) {
            let app = Application("Terminal");
            if (!app.running()) {
                app.activate();
            }
            app.doScript(argv[0]);
        }
    "#;

    pub fn run_in_new_window(script: &OsStr) -> Result<(), String> {
        super::run_jxa(OsStr::new(JXA_RUN), script)
    }
}

// iTerm2
mod iterm {
    use std::ffi::OsStr;

    const JXA_RUN: &str = r#"
        function run(argv) {
            let app = Application("iTerm");
            if (!app.running()) {
                app.activate();
            }
            let window = app.createWindowWithDefaultProfile({});
            window.currentSession().write({"text": argv[0]});

        }
    "#;

    pub fn run_in_new_window(script: &OsStr) -> Result<(), String> {
        super::run_jxa(OsStr::new(JXA_RUN), script)
    }
}


