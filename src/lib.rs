// TODO: check out clap_complete (and other related projects listed in the clap docs)
//   - clap_mangen
// TODO: custom colors
//   - TODO: add rgb color support instead of just ANSI
//   - TODO: add a default location to look for config file if -c is not specified
mod config_file;
mod utils;

use crate::config_file::get_config;
use crate::config_file::ConfigColor;
use crate::utils::{is_timestamp, replace_trailing_cr_with_crlf};
use clap::{Parser, ValueHint};
use colored::{ColoredString, Colorize};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use std::{env, io};

type CustomResult<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(
        help = "File to watch, e.g. be path/to/Import.log. Either specify the path here or via the --path arg. Leave both empty to use current directory.",
        conflicts_with = "use_docs_dir",
        conflicts_with = "path",
        value_names = &["PATH"],
        value_hint = ValueHint::FilePath,
    )]
    path_unnamed: Option<String>,

    #[arg(
        long = "path",
        short = 'p',
        help = "File to watch, e.g. path/to/Import.log. Either specify the path here or via the [PATH] arg. Leave both empty to use current directory.",
        required = false,
        conflicts_with = "use_docs_dir",
        conflicts_with = "path_unnamed",
        value_hint = ValueHint::FilePath,
    )]
    path: Option<String>,

    #[arg(
        long = "docs-dir",
        short = 'd',
        help = "Open log from your local Documents directory (default location when working with hosted files) instead of custom path"
    )]
    use_docs_dir: bool,

    #[arg(long, help = "Don't watch for changes, just print once")]
    no_watch: bool,

    #[arg(long, help = "Don't print color")]
    no_color: bool,

    #[arg(
        long,
        short,
        help = "Only print errors, can be combined with warnings-only"
    )]
    errors_only: bool,

    #[arg(
        long,
        short,
        help = "Only print warnings, can be combined with errors-only"
    )]
    warnings_only: bool,

    #[arg(
        long = "config",
        short = 'c',
        help = "Path to config file for customizing colors",
        value_name = "PATH"
    )]
    config_path: Option<String>,
    // how should filter be passed in? what if we want multiple filters?
    //   - maybe some basic filters and a regex option?
    #[arg(long, short, help = "Print a separator between each import operation")]
    separator: bool,
}

struct ImportLogLine {
    timestamp: String,
    filename: String,
    code: String,
    message: String,
}
impl ImportLogLine {
    fn contains_warning_text(&self) -> bool {
        self.code.eq("0")
            && (self.message.ends_with("already exists.")
                || self
                    .message
                    .ends_with("created and imported automatically."))
    }
    fn is_operation_start(&self) -> bool {
        self.code.eq("0") && self.message.ends_with(" started")
    }
}
impl ToString for ImportLogLine {
    fn to_string(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.timestamp, self.filename, self.code, self.message
        )
    }
}

fn is_header(line: &str) -> bool {
    line.ends_with("Timestamp\tFilename\tError\tMessage")
}

enum LineType {
    Success(ImportLogLine),
    Error(ImportLogLine),
    Warning(ImportLogLine),
    Header(ImportLogLine),
    Other(String),
}
impl LineType {
    fn is_header(&self) -> bool {
        matches!(self, LineType::Header(_))
    }
    fn is_error(&self) -> bool {
        matches!(self, LineType::Error(_))
    }
    fn is_warning(&self) -> bool {
        matches!(self, LineType::Warning(_))
    }
}
impl ToString for LineType {
    fn to_string(&self) -> String {
        match self {
            LineType::Error(line) => line.to_string(),
            LineType::Header(line) => line.to_string(),
            LineType::Other(line) => line.to_string(),
            LineType::Success(line) => line.to_string(),
            LineType::Warning(line) => line.to_string(),
        }
    }
}
fn parse_line(line: &str) -> LineType {
    let v = line.splitn(4, '\t').collect::<Vec<&str>>();
    let timestamp = v.first().unwrap_or(&"").to_string();

    let header = is_header(line);
    if is_timestamp(&timestamp) || header {
        let filename = v.get(1).unwrap_or(&"").to_string();
        let code = v.get(2).unwrap_or(&"").to_string();
        let message = v.get(3).unwrap_or(&"").to_string();
        let mut line = ImportLogLine {
            timestamp,
            filename,
            code,
            message,
        };
        if header {
            LineType::Header(line)
        } else if line.code == "0" {
            return if line.contains_warning_text() {
                LineType::Warning(line)
            } else {
                LineType::Success(line)
            };
        } else {
            replace_trailing_cr_with_crlf(&mut line.message);
            LineType::Error(line)
        }
    } else {
        LineType::Other(line.to_string())
    }
}

#[derive(Debug)]
enum PathType {
    CustomPath(PathBuf),
    CurrentDir(PathBuf),
    DocsDir(PathBuf),
}
impl PathType {
    fn message(&self) -> String {
        match self {
            PathType::CurrentDir(path) => format!("Using current directory: {}", path.display()),
            PathType::DocsDir(path) => format!("Using documents directory: {}", path.display()),
            _ => "".to_string(),
        }
    }
    fn path(&self) -> &PathBuf {
        match self {
            PathType::CustomPath(path) => path,
            PathType::CurrentDir(path) => path,
            PathType::DocsDir(path) => path,
        }
    }
}
fn get_path(args: &Args) -> CustomResult<PathType> {
    match args {
        Args {
            path: Some(path), ..
        } => Ok(PathType::CustomPath(path.into())),

        Args {
            path_unnamed: Some(path),
            ..
        } => Ok(PathType::CustomPath(path.into())),

        Args {
            use_docs_dir: true, ..
        } => {
            let mut path = dirs::document_dir().ok_or("couldn't find documents directory")?;
            path.push("Import.log");
            if path.exists() {
                Ok(PathType::DocsDir(path))
            } else {
                Err("couldn't find Import.log in the documents directory. See --help".into())
            }
        }

        _ => {
            let mut path = env::current_dir().or(Err("couldn't find current directory"))?;
            path.push("Import.log");
            if path.exists() {
                Ok(PathType::CurrentDir(path))
            } else {
                Err("couldn't find Import.log in the current directory. See --help".into())
            }
        }
    }
}

fn get_default_colorizer(
    config_color: ConfigColor,
    default_foreground: String,
) -> impl Fn(&str) -> ColoredString {
    move |line: &str| {
        let foreground = if config_color.foreground.is_empty() {
            default_foreground.as_str()
        } else {
            config_color.foreground.as_str()
        };
        let background = if config_color.background.is_empty() {
            ""
        } else {
            config_color.background.as_str()
        };

        let mut res = line.color(foreground);
        if !background.is_empty() {
            res = res.on_color(background)
        }
        res
    }
}

fn colorize_columns(
    line: &ImportLogLine,
    timestamp_colorizer: &impl Fn(&str) -> ColoredString,
    filename_colorizer: &impl Fn(&str) -> ColoredString,
    error_colorizer: &impl Fn(&str) -> ColoredString,
    message_colorizer: &impl Fn(&str) -> ColoredString,
) -> [ColoredString; 4] {
    let ts = timestamp_colorizer(&line.timestamp);
    let filename = filename_colorizer(&line.filename);
    let error = error_colorizer(&line.code);
    let msg = message_colorizer(&line.message);
    [ts, filename, error, msg]
}

pub fn run() -> CustomResult {
    #[cfg(target_os = "windows")]
    colored::control::set_virtual_terminal(true).unwrap();

    let args = Args::parse();
    let path_type = get_path(&args)?;

    let config = get_config(args.config_path.as_deref())?;

    let path = path_type.path();
    let msg = path_type.message();
    if !msg.is_empty() {
        let msg = if args.no_color {
            msg
        } else {
            msg.green().bold().underline().to_string()
        };
        println!("{}", msg);
    }

    // get colorizer for each field:
    let timestamp_colorizer = get_default_colorizer(config.colors.timestamp, "cyan".to_string());
    let filename_colorizer = get_default_colorizer(config.colors.filename, "green".to_string());
    let error_colorizer = get_default_colorizer(config.colors.error, "bright magenta".to_string());
    let message_colorizer = get_default_colorizer(config.colors.message, "bright blue".to_string());

    // closure/fn to handle each line
    let handle_line = |line: &str| {
        let line = parse_line(line);
        let show_line = line.is_header()
            || (args.errors_only && line.is_error())
            || (args.warnings_only && line.is_warning())
            || (!args.errors_only && !args.warnings_only);
        if !show_line {
            return;
        };
        if args.no_color {
            println!("{}", line.to_string());
        } else {
            match line {
                LineType::Success(line) => {
                    let res = colorize_columns(
                        &line,
                        &timestamp_colorizer,
                        &filename_colorizer,
                        &error_colorizer,
                        &message_colorizer,
                    );
                    let [a, b, c, d] = res;
                    if args.separator && line.is_operation_start() {
                        println!(
                            "-----------------------------------------------------------------"
                        );
                    }
                    println!("{} {} {} {}", a, b, c, d);
                }
                LineType::Error(line) => {
                    println!(
                        "{} {} {} {}",
                        line.timestamp.bright_white().on_red(),
                        line.filename.bright_white().on_red(),
                        line.code.bright_white().on_red(),
                        line.message
                    );
                }
                LineType::Warning(line) => {
                    println!(
                        "{} {} {} {}",
                        line.timestamp.black().on_yellow(),
                        line.filename.black().on_yellow(),
                        line.code.black().on_yellow(),
                        line.message
                    )
                }
                LineType::Header(line) => {
                    let res = colorize_columns(
                        &line,
                        &timestamp_colorizer,
                        &filename_colorizer,
                        &error_colorizer,
                        &message_colorizer,
                    );
                    let [a, b, c, d] = res.map(|s| s.underline());
                    println!("{} {} {} {}", a, b, c, d);
                }
                LineType::Other(line) => println!("{}", line.to_string()),
            }
        }
    };

    let file = File::open(path).map_err(|e| format!("couldn't open '{:?}', {}", path, e))?;
    let mut reader = io::BufReader::new(&file);
    let mut buf = String::new();

    // read the initial file content
    reader.read_to_string(&mut buf).unwrap();
    buf.lines().for_each(handle_line);
    if args.no_watch {
        return Ok(());
    }

    let mut pos = buf.len() as u64;
    // Watch the file for changes
    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(100), None, tx).unwrap();
    debouncer
        .watcher()
        .watch(path, RecursiveMode::NonRecursive)
        .unwrap();

    // Listen for messages passed from the debouncer thread
    for res in rx {
        match res {
            Ok(_) => {
                reader.seek(SeekFrom::Start(pos)).unwrap();
                pos = file.metadata().unwrap().len();

                buf.clear();
                reader.read_to_string(&mut buf).unwrap();
                buf.lines().for_each(handle_line);
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

// ###################################################################################
// Tests
// ###################################################################################
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line() {
        let ts = "2020-05-01 12:00:00.000";
        let filename = "/Users/username/Downloads/Import.log";
        let code = "0";
        let message = "Imported file";

        // regular
        let line = parse_line(format!("{}\t{}\t{}\t{}", ts, filename, code, message).as_str());
        let LineType::Success(val) = line else {
            panic!("expected regular line");
        };
        assert!(
            val.timestamp == ts
                && val.filename == filename
                && val.code == code
                && val.message == message
        );

        // error
        let code = "123";
        let line = parse_line(format!("{}\t{}\t{}\t{}", ts, filename, code, message).as_str());
        let LineType::Error(val) = line else {
            panic!("expected Error line");
        };
        assert!(
            val.timestamp == ts
                && val.filename == filename
                && val.code == code
                && val.message == message
        );

        // warning
        let code = "0";
        let message = "something something ... already exists.";
        let line = parse_line(format!("{}\t{}\t{}\t{}", ts, filename, code, message).as_str());
        let LineType::Warning(val) = line else {
            panic!("expected Warning line");
        };
        assert!(
            val.timestamp == ts
                && val.filename == filename
                && val.code == code
                && val.message == message
        );

        // other
        let string = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non nibh at neque vehicula accumsan quis hendrerit ligula. Integer vestibulum justo dolor, sit amet maximus mi euismod sed. Praesent rhoncus eros sed orci imperdiet sollicitudin. Proin ornare erat";
        let LineType::Other(_) = parse_line(string) else {
            panic!("expected Other line");
        };

        // header
        let line = parse_line("lkjflkjf Timestamp\tFilename\tError\tMessage");
        let LineType::Header(_) = line else {
            panic!("expected Error line");
        };
    }

    #[test]
    fn parse_line_should_handle_trailing_crs() {
        let ts = "2020-05-01 12:00:00.000";
        let filename = "/Users/username/Downloads/Import.log";
        let code = "123";
        let message = "Imported file\ranother line\r a third line\r";
        let line = parse_line(format!("{}\t{}\t{}\t{}", ts, filename, code, message).as_str());
        let LineType::Error(val) = line else {
            panic!("expected Error line");
        };
        assert_eq!(val.timestamp, ts);
        assert_eq!(val.filename, filename);
        assert_eq!(val.code, code);
        assert_eq!(
            val.message,
            "Imported file\r\nanother line\r\n a third line\r"
        );
    }

    #[test]
    fn test_is_header() {
        assert!(is_header(
            "anythinghere-- Timestamp\tFilename\tError\tMessage"
        ));
        let line = "hello world";
        assert!(!is_header(line));
    }
}
