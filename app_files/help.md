
- **Command:** The terminal program to execute. The files, space-delimited, will be appended to this string and passed to the terminal. Only Bash-compatible commands are supported at the moment (due to the quoting required for the command and passed file paths).
- **Documents:** Document types to support opening. This will control which files your shim app appears in the `Open With` menu for. Other files will still be openable with `Open With` -> `Other...` (perhaps enabling `All Applications`). NB: UTI is [Uniform Type Identifier](https://developer.apple.com/documentation/uniformtypeidentifiers).
    - _Text Files_: Support opening text files (UTIs: `public.text`, `public.data`)
    - _All Documents_: Support opening all documents (UTIs: `public.content`, `public.data`)
    - _Specific UTIs_: enter a comma-delimited list of UTIs to support.
    - _Specific Extensions_: enter a comma-delimited list of extensions to support. Wildcard (`*`) is no extension.
- **Terminal:** Select desired terminal application. Currently supported are Terminal.app and iTerm2. To try to use another terminal, select `Generic`, and enter the terminal's name. An attempt will be make to control the terminal by sending keystrokes (best effort). Permission must first be given for your shim app to control your computer in `System Preferences` -> `Privacy and Security` -> `Accessbility`.
- **Open Files: () Together, () Individually:** If multiple files are opened simultaneously, should they all be passed to a single instantiation to the command (space-delimited), or should each open in it's own window? Note that this only applies to files opened at one time - files opened thereafter will currently always open in new windows.

A custom icon can also be chosen with "Select Icon...". Currently, the GUI only supports `png`s, but the CLI can be used to choose any format MacOS supports, including `icns`.

Then click `Save As…`, provide a file name and directory, and click `Save`. You can then set your shim app as the `Open With` handler, or launch it to provide a draggable target in the Dock (no windows will appear after being launched, and launching the shim app ahead of time isn't necessary).
