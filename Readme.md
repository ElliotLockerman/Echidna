
![Echidna Header](media/header.png)


Echidna is a Mac app for generating "shim applications" that allow opening files with terminal programs by double clicking on the files' icons. For example, you could generate a shim app to open double-clicked source files in `(n)vim` or `emacs`. Files can also be opened with the selected terminal application by dragging to the shim's icon in the Finder or Dock or selecting the shim app from the `Open With` menu after right-clicking on the file icon. Echidna's name, like its functionality, is inspired by [Platypus](https://sveinbjorn.org/platypus) ([Github](https://github.com/sveinbjornt/Platypus)), a wonderful Mac app for wrapping scripts in GUIs.

![Demo Screenshot](media/opening_file.gif)

## Usage

![UI Screenshot](media/screenshot_0.png)

After launching Echidna, first fill out the fields:

- **Command**: The terminal program to execute. The files, space-delimited, will be appended to this string and passed to the terminal.
- **Extensions**: Optionally add extensions for the shim app to support. Adding extensions makes your shim app appear in the `Open With` menu for files with supported extensions, but will prevent files with other extensions from being dragged to your shim app. Files with other extensions can still be opened by double-clicking if your shim app is selected in the `Open With` -> `Other...` dialog (You may have to select `All Applications` in the `Enable` drop-down)
- **Open Files: () Together, () Individually**: If multiple files are opened simultaneously, should they all be passed to a single instantiation to the command (space-delimited), or should each open in it's own window?

Then click `Save As..`, provide a file name and directory, and click `Save`. You can then set your shim app as the `Open With` handler, or launch it to provide a draggable target in the dock bar (no windows will appear after being launched, and launching isn't necessary for other use pattern).


## Repo Structure

- `echidna-util`: `lib`. Small pieces of functionality that have no dependencies within Echidna.
- `echidna-shim`: `bin`. The binary that runs within the generated shim app, receiving the double-clicked files and launching the terminal session.
- `echidna-lib`: `lib`. The library with the core Echidna functionality of generating specialized shim apps. Depends on `echidna-util` (as a library in the traditional manner), and `bin` (compiled in as a `CONST` variable). This dependency on a binary is why `make` is used rather than `cargo build`: `cargo` does not (yet) support binary dependencies, so if `echidna-shim` is not manually rebuilt before each `echidna-lib` build, an out-of-date `echidna-shim` might be used.
- `echidna-cli`: `bin`. A command line tool to generate shim apps. Essentially a thin wrapper around `echidna-lib`. By default, `echidna-cli` looks for an `echidna-shim` binary in the same directory, but this can be overwritten with a command-line flag. `echidna-cli` _should not_ be run with `cargo run`, as Cargo is not aware of the dependency between `echidna-cli` and `echidna-shim`, and a stale version of `echidna-shim` may be used.
- `echidna`: `bin`. A GUI tool to generate shim apps. Essentially a (slightly less) thin wrapper around `echidna-lib`.


## Building

**TLDR: Run `./build.sh [--debug | --release]`**

`cargo build` builds the Rust binaries, but to run `echidna-cli` and build the `Echidna.app` Mac app bundle, you have to run `cargo build --workspace` and `./echidna/scripts/make-app.sh` (which is what `./build.sh` does). `--workspace` is required because `echidna-cli` and `Echidna.app` have a runtime dependency on `echidna-shim`, but Cargo doesn't currently doesn't support this kind of dependency, so running `cargo build` may result in a stale version of `echidna-shim` being used. `Echidna.app` will be built in `target/{BUILD_MODE}/Echidna.app`.
