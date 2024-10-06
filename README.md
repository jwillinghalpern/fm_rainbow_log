# FM Rainbow Log (fmrl)

`fmrl` (pronounced like Earl with a "fmuh") is a cross-platform command line tool to:

- 🥸 watch FileMaker Import.log files for changes
- 🦄 colorize the output
- 🚨 highlight errors and warnings.
- 🔔 display notifications and/or beep when issues are detected

It displays real-time updates for both custom Import.log paths as well as the default (Documents) location when working with hosted fmp12 files.

![fmrl demo](./readme-files/example.png)

## Motivation

FileMaker's Import.log file is helpful when importing/copying between files, but it's difficult to parse visually. Errors and warnings get buried, and columns don't align. Plus, the default macOS log viewer, Console.app wraps lines unnecessarily 🤨.

Furthermore, FileMaker calcs don't play nice with other CLI utilities like `tail` because they sometime use non-standard `\r` line endings. `fmrl` appends linefeeds as needed so messages aren't truncated when printing in the terminal.

## Installation

### 🦀 Via cargo (recommended)

```bash
cargo install --git https://github.com/jwillinghalpern/fm_rainbow_log.git
```

To update fmrl, re-run the above command any time. If you get a warning that `rustc` is outdated, simply run either `rustup update` or `rustup update stable` then try installing `fmrl` again.

If you don't have cargo, [follow this one easy step](https://doc.rust-lang.org/cargo/getting-started/installation.html)

Please note, [installing the rust compiler on mac/linux](https://doc.rust-lang.org/book/ch01-01-installation.html#installing-rustup-on-linux-or-macos) also involves installing a linker and/or C compiler, which is most easily done via `xcode-select --install`. Please see the previous link for installation instructions.

### 🤖 Via pre-compiled binary

I prefer using cargo (described above), because it streamlines updates and avoids permission issues. But pre-compiled binaries are also available.

1. Copy the latest `fmrl` binary [from here](https://github.com/jwillinghalpern/fm_rainbow_log/releases) to a directory in your PATH. For example, `/usr/local/bin` on macOS.
   - To see the folders in your PATH, run this in your terminal: `echo $PATH | sed -E 's/:/\n/g'`
2. Restart your terminal and type `fmrl --help`.

NOTE: On macOS, the first time you run the program you'll encounter a security warning. [See here](./readme-files/macos-security.md)

## 🏎️ Basic Usage

watch Import.log in current directory:

```bash
fmrl
```

show help (it's helpful)

```bash
fmrl --help
```

watch Import.log in your local Documents directory (default location when working with hosted files):

```bash
fmrl --docs-dir
# fmrl -d
```

### Customize with `config.json` (recommended)

To customize colors and default options, create config/json file (the location to store it is described below). All keys are optional, including nested keys.

```json5
// this file supports json5 syntax, so you can use comments and trailing commas
{
  "show_separator": false,
  "use_documents_directory": false,
  "errors_only": false,
  "warnings_only": false,

  "show_notifications": false,

  "beep": false,
  "beep_volume": 1.0,
  "beep_path": "/System/Library/Sounds/Tink.aiff",

  // error_rules fields:
  //   - action: "quiet" or "ignore"
  //     - "quiet" : still highlight the error red, but don't produce desktop notification
  //     - "ignore" : don't even highlight the error
  //   - error_code (optional): the error code to match
  //   - message_contains (optional): String or Array of strings. the text to match. If an array is passed, every substring must be present in the line to match.
  //   - message_starts_with (optional): the text to match
  //   - message_ends_with (optional): the text to match
  //   - location_contains (optional): String or Array of strings. the text to match. If an array is passed, every substring must be present in the line to match.
  //   - location_starts_with (optional): the text to match
  //   - location_ends_with (optional): the text to match

  //   NOTE: ALL fields fields must be satisfied (Like an AND operator) for a rule to trigger `action`. Therefore, fewer fields set will have a broader effect.

  "error_rules": [
    { "error_code": "123", "message_contains": "foo", "action": "quiet" },
    { "message_contains": ["arrays", "work"], "action": "quiet" },
    { "location_contains": ["location", "too"], "action": "quiet" },
    { "error_code": "234", "action": "ignore" }
    { "message_contains": "I'm not an important error", "action": "ignore" }
  ],

  // you can omit nested keys if you want
  "colors": {
    "timestamp": {
      "foreground": "bright white",
      "background": "magenta"
    },
    "filename": {
      "foreground": "black",
      // "background": "cyan"
    },
    "error": {
      "foreground": "bright white",
      "background": "bright green"
    },
    "message": {
      "foreground": "bright white",
      "background": "black"
    }
  },

  // (DEPRECATED) this option will be removed in the future. Please use error_rules instead.
  "quiet_errors": "3702, 1234"
}
```

#### Where to store the config

either:

1. pass in the path with the `-c/--config` option:

   ```bash
   fmrl -c path/to/config.json
   ```

2. (recommended) save the file in this location, and then fmrl will use that by default:

   To see the location `fmrl` will look for a default config file, run this command:

   ```bash
   fmrl --print-config-path
   ```

Here are the default locations

- Mac: `$HOME/Library/Application Support/fm_rainbow_log/config.json`
  - example: `/Users/Alice/Library/Application Support/fm_rainbow_log/config.json`
- Windows: `{FOLDERID_RoamingAppData}\fm_rainbow_log\config.json`
  - example: `C:\Users\Alice\AppData\Roaming\fm_rainbow_log\config.json`

_If you have a default config.json, you can override it by passing a different path to the `--config` option._

## 🖐️ Handy Opener Tool for MacOS!

When working with local fmp12 files, please also see [this nice opener tool](https://github.com/DanShockley/FM_Rainbow_Log-Opener-applet). You can copy it into a local project folder and double click any time to open `fmrl` for that project.

## 🎨 Colors

`fmrl` supports both ANSI and truecolor. ANSI colors are the standard 16 colors supported by most terminals, whereas truecolor is a newer standard. Some terminals including macOS Terminal.app _do not_ support truecolor, but modern terminals like iTerm2, Alacritty, and Warp do. You can define truecolors as rgb or hex (see below).

### ANSI format

| color   | bright version |
| ------- | -------------- |
| black   | bright black   |
| red     | bright red     |
| green   | bright green   |
| yellow  | bright yellow  |
| blue    | bright blue    |
| magenta | bright magenta |
| cyan    | bright cyan    |
| white   | bright white   |

### Truecolor format

rgb: `rgb(255, 0, 255)`

hex: `#ff00ff`

## 👽 Additional Usage Examples

print a separator between each import operation:

```bash
fmrl -s
```

watch for only errors and warnings:

```bash
fmrl  --errors-only --warnings-only
# fmrl -e -w
# fmrl -ew
```

show desktop notifications for errors and warnings:

```bash
fmrl --notifications
```

play sound! (mac only)

```bash
fmrl --beep
# or make it fancier!
fmrl --notifications --beep --beep-volume 0.8 --beep-path /System/Library/Sounds/Frog.aiff
```

specify custom config/colors file:

```bash
fmrl -c path/to/config.json
```

don't watch for changes, just print the log once:

```bash
fmrl --no-watch
```

generate an auto-completion script (store somewhere in your $fpath, see below for mac example):

```bash
# zsh example. Omit "zsh" to see shell options.
fmrl --completion zsh

# here's where I put it!
fmrl --completion zsh > ~/.oh-my-zsh/completions/_fm_rainbow_log
```

## 👩‍💻 Development/contribution

Fork to your own Github account, clone this repo to your desktop, cd to the directory, and run `cargo run` to test in debug mode. If you are planning a big feature or change, please open an issue first to discuss. It's best to create a new branch for the specific feature/issue you're working on.

## 📓 Notes

- Most terminals let you customize the ANSI colors, so feel free to tweak the appearance to your liking!
- On Windows I've only tested PowerShell. There are certain cases where the color escape sequences don't display properly, and show garbled text. I'm not sure how to handle every edge case (please submit suggestions/pull requests if you do).
