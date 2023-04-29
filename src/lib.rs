// TODO: check out clap_complete (and other related projects listed in the clap docs)
//   - clap_mangen
// TODO: custom colors
//   - for modern terminals, we can use truecolor. (Maybe we only support it for truecolor)
mod utils;

use clap::Parser;
use colored::Colorize;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use std::{env, io};
use utils::{is_timestamp, replace_trailing_cr_with_crlf};

type CustomResult<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(
        help = "File to watch, e.g. be path/to/Import.log. Either specify the path here or via the --path arg. Leave both empty to use current directory.",
        conflicts_with = "use_docs_dir",
        conflicts_with = "path",
        value_names = &["PATH"],
    )]
    path_unnamed: Option<String>,

    #[arg(
        long = "path",
        short = 'p',
        help = "File to watch, e.g. path/to/Import.log. Either specify the path here or via the [PATH] arg. Leave both empty to use current directory.",
        required = false,
        conflicts_with = "use_docs_dir",
        conflicts_with = "path_unnamed"
    )]
    path: Option<String>,

    #[arg(
        long = "docs-dir",
        short = 'd',
        help = "Open log from Documents directory (default location for hosted files) instead of custom path"
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
    // how should filter be passed in? what if we want multiple filters?
    //   - maybe some basic filters and a regex option?
}

trait ToColorString {
    fn to_color_string(&self) -> String;
}

struct ImportLogLine {
    timestamp: String,
    filename: String,
    code: String,
    message: String,
}
impl ImportLogLine {
    fn is_warning(&self) -> bool {
        self.code.eq("0") && self.message.ends_with("already exists.")
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

fn colorize_default(line: &ImportLogLine) -> String {
    format!(
        "{}\t{}\t{}\t{}",
        line.timestamp.green(),
        line.filename.cyan(),
        line.code.magenta(),
        line.message.blue()
    )
}
fn colorize_header(line: &ImportLogLine) -> String {
    format!(
        "{}\t{}\t{}\t{}",
        line.timestamp.green().underline(),
        line.filename.cyan().underline(),
        line.code.magenta().underline(),
        line.message.blue().underline()
    )
}
fn colorize_error(line: &ImportLogLine) -> String {
    format!(
        "{}\t{}\t{}\t{}",
        line.timestamp.bright_white().on_red(),
        line.filename.bright_white().on_red(),
        line.code.bright_white().on_red(),
        line.message
    )
}
fn colorize_warning(line: &ImportLogLine) -> String {
    format!(
        "{}\t{}\t{}\t{}",
        line.timestamp.black().on_yellow(),
        line.filename.black().on_yellow(),
        line.code.black().on_yellow(),
        line.message
    )
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
impl ToColorString for LineType {
    fn to_color_string(&self) -> String {
        match self {
            LineType::Error(line) => colorize_error(line),
            LineType::Header(line) => colorize_header(line),
            LineType::Other(line) => line.to_string(),
            LineType::Success(line) => colorize_default(line),
            LineType::Warning(line) => colorize_warning(line),
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
            return if line.is_warning() {
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
fn get_path(config: &Config) -> CustomResult<PathType> {
    match config {
        Config {
            path: Some(path), ..
        } => Ok(PathType::CustomPath(path.into())),

        Config {
            path_unnamed: Some(path),
            ..
        } => Ok(PathType::CustomPath(path.into())),

        Config {
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

pub fn run() -> CustomResult {
    #[cfg(target_os = "windows")]
    colored::control::set_virtual_terminal(true).unwrap();

    let config = Config::parse();
    let path_type = get_path(&config)?;

    let path = path_type.path();
    let msg = path_type.message();
    if !msg.is_empty() {
        let msg = if config.no_color {
            msg
        } else {
            msg.green().bold().underline().to_string()
        };
        println!("{}", msg);
    }

    // closure/fn to handle each line
    let handle_line = |line: &str| {
        let line = parse_line(line);
        let show_line = line.is_header()
            || (config.errors_only && line.is_error())
            || (config.warnings_only && line.is_warning())
            || (!config.errors_only && !config.warnings_only);
        if !show_line {
            return;
        };
        if config.no_color {
            println!("{}", line.to_string());
        } else {
            println!("{}", line.to_color_string());
        }
    };

    let file = File::open(path).map_err(|e| format!("couldn't open '{:?}', {}", path, e))?;
    let mut reader = io::BufReader::new(&file);
    let mut buf = String::new();

    // read the initial file content
    reader.read_to_string(&mut buf).unwrap();
    buf.lines().for_each(handle_line);
    if config.no_watch {
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
