use crate::bail;
use crate::config::{Config, TerminalApp};

use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::process::{Command, Stdio};

use indexmap::{indexmap, IndexMap};
use lazy_static::lazy_static;

type RunInNewWindow = fn(config: &Config, bash: &OsStr) -> Result<(), String>;

lazy_static! {
    // TERMINALS and TERM_ORDER must have exactly the same keys!!!
    static ref TERMINALS: IndexMap<String, RunInNewWindow> = indexmap! {
        "Terminal.app".into() => terminal_dot_app::run_in_new_window as RunInNewWindow,
        "iTerm2".into() => iterm::run_in_new_window as RunInNewWindow,
    };
}

pub fn run_in_new_window(config: &Config, bash: &OsStr) -> Result<(), String> {
    match &config.terminal {
        TerminalApp::Supported(name) => match TERMINALS.get(name) {
            Some(fun) => fun(config, bash),
            None => Err(format!(
                "Terminal {} is not supported",
                &config.terminal.name()
            )),
        },
        TerminalApp::Generic(_) => generic::run_in_new_window(config, bash),
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

fn run_jxa(jxa: &OsStr, term: &OsStr, arg: &OsStr) -> JxaResult {
    let cmd = "osascript";
    let args = [OsStr::new("-lJavaScript"), OsStr::new("-"), term, arg];

    let mut child = Command::new::<&OsStr>(cmd.as_ref())
        .args(args)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| "Run error: ".to_owned() + e.to_string().as_str() + "\n")?;

    /* scope to close stdin and unblock osascript */
    {
        let mut child_stdin = child
            .stdin
            .take()
            .ok_or("Couldn't get child's stdin".to_owned())?;
        child_stdin
            .write(jxa.as_bytes())
            .map_err(|e| format!("Couldn't write to child's stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Error waiting on child: {e}"))?;
    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr);
        bail!(msg);
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

// MacOS's built-in terminal
mod terminal_dot_app {
    use crate::config::Config;
    use std::ffi::OsStr;

    const JXA_RUN: &str = r#"
        function run(argv) {
            if (argv.length !== 2) {
                console.log("Expected exactly 2 arguments");
                return;
            }

            let app = Application("Terminal");
            if (!app.running()) {
                app.activate();
            }
            app.doScript(argv[1]);
        }
    "#;

    pub fn run_in_new_window(_: &Config, script: &OsStr) -> Result<(), String> {
        super::run_jxa(OsStr::new(JXA_RUN), OsStr::new(""), script)
    }
}

// iTerm2
mod iterm {
    use crate::config::Config;
    use std::ffi::OsStr;

    const JXA_RUN: &str = r#"
        function run(argv) {
            if (argv.length !== 2) {
                console.log("Expected exactly 2 arguments");
                return;
            }

            let app = Application("iTerm");
            if (!app.running()) {
                app.activate();
            }
            let window = app.createWindowWithDefaultProfile({});
            window.currentSession().write({"text": argv[1]});

        }
    "#;

    pub fn run_in_new_window(_: &Config, script: &OsStr) -> Result<(), String> {
        super::run_jxa(OsStr::new(JXA_RUN), OsStr::new(""), script)
    }
}

mod generic {
    use crate::config::Config;
    use std::ffi::OsStr;

    const JXA_RUN: &str = r#"
    function run(argv) {
        if (argv.length !== 2) {
            console.log("Expected exactly 2 arguments");
            return;
        }

        let app = Application(argv[0]);
        let was_running = app.running();
        app.activate();

        let events = Application("System Events");
        if (was_running) {
            events.keystroke("n", {"using": "command down"});
        }
        delay(0.25);
        events.keystroke(argv[1]);
    }
    "#;

    pub fn run_in_new_window(config: &Config, script: &OsStr) -> Result<(), String> {
        // Assuming OsStr(ing) is backwards-compatible with ascii...
        let mut script = script.to_owned();
        script.push("\n");
        super::run_jxa(
            OsStr::new(JXA_RUN),
            OsStr::new(config.terminal.name()),
            &script,
        )
    }
}
