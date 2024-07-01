
![Echidna Header](media/header.png)


Echidna is a Mac app for generating "shim applications" that allow opening files with terminal programs by double clicking on the files' icons. For example, you could generate a shim app to open double-clicked source files in `(n)vim` or `emacs`. Files can also be opened with the selected terminal application by dragging to the shim's icon in the Finder or Dock or selecting the shim app from the `Open With` menu after right-clicking on the file icon. Echidna's name, like its functionality, is inspired by [Platypus](https://sveinbjorn.org/platypus) ([Github](https://github.com/sveinbjornt/Platypus)), a wonderful Mac app for wrapping scripts in GUIs.

<div align="center">
    <img src="media/shim_demo.gif" alt="Demo Animation" text-align="center">
</div>

## Usage

![UI Screenshot](media/screenshot_0.png)

After launching Echidna, first fill out the fields:

- **Command:** The terminal program to execute. The files, space-delimited, will be appended to this string and passed to the terminal. Only Bash-compatible commands are supported at the moment (due to the quoting required for the command and passed file paths).
- **Documents:** Document types to support opening. This will control which files your shim app appears in the `Open With` menu for. Other files will still be openable with `Open With` -> `Other...` (perhaps enabling `All Applications`). NB: UTI is [Uniform Type Identifier](https://developer.apple.com/documentation/uniformtypeidentifiers).
    - _Text Files_: Support opening text files (UTIs: `public.text`, `public.data`)
    - _All Documents_: Support opening all documents (UTIs: `public.content`, `public.data`)
    - _Specific UTIs_: enter a comma-delimited list of UTIs to support.
    - _Specific Extensions_: enter a comma-delimited list of extensions to support. Wildcard (`*`) is no extension.
- **Terminal:** Select desired terminal application. Currently supported are Terminal.app and iTerm2. To try to use another terminal, select `Generic`, and enter the terminal's name. An attempt will be make to control the terminal by sending keystrokes (best effort). Permission must first be given for your shim app to control your computer in `System Preferences` -> `Privacy and Security` -> `Accessbility`.
- **Open Files: () Together, () Individually:** If multiple files are opened simultaneously, should they all be passed to a single instantiation to the command (space-delimited), or should each open in it's own window? Note that this only applies to files opened at one time - files opened thereafter will currently always open in new windows.

A custom icon can also be chosen with "Select Icon...".

Then click `Save As…`, provide a file name and directory, and click `Save`. You can then set your shim app as the `Open With` handler, or launch it to provide a draggable target in the Dock (no windows will appear after being launched, and launching the shim app ahead of time isn't necessary).


## Repo Structure

- `echidna-shim`: `bin`. The binary that runs within the generated shim app, receiving the double-clicked files and launching the terminal session.
- `echidna_lib`: `lib`. The library with the core Echidna functionality of generating specialized shim apps.
- `echidna-cli`: `bin`. A command line tool to generate shim apps. Essentially a thin wrapper around `echidna-lib`. By default, `echidna-cli` looks for an `echidna-shim` binary in the same directory, but this can be overwritten with a command-line flag. `echidna-cli` _should not_ be run with `cargo run`, as Cargo is not aware of the dependency between `echidna-cli` and `echidna-shim`, and a stale version of `echidna-shim` may be used.
- `echidna`: `bin`. A GUI tool to generate shim apps. Essentially a (slightly less) thin wrapper around `echidna-lib`.


## Building

**TLDR: Run `./build.sh [--debug | --release]`**

`build.sh` runs `cargo build --all`, then `scripts/make-app.sh`, which builds `Echidna.app` (in `target/{BUILD_MODE}/Echidna.app`), a Mac app bundle that includes `echidana` and `echidna-shim`.

`cargo run` _should not_ be used, see `echidna-cli` above.
