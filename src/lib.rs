// TODO: check out clap_complete (and other related projects listed in the clap docs)
//   - clap_mangen
//   - clap_mangen
// TODO: custom colors
// TODO: highlight warnings
//   - change default code color away from yellow and use that for warnings
mod utils;

use clap::Parser;
use colored::Colorize;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
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
    // TODO: make file optional and default to Import.log in the OS's documents folder
    #[arg(
        help = "The file to watch, should probably be path/to/Import.log. you can specify the file here or with the --path flag",
        conflicts_with = "path",
        value_names = &["PATH"]

    )]
    path_unnamed: Option<String>,

    #[arg(
        long = "path",
        short = 'p',
        help = "The file to watch, should probably be path/to/Import.log",
        required = false,
        conflicts_with = "path_unnamed"
    )]
    path: Option<String>,

    #[arg(
        long = "docs",
        short = 'd',
        help = "Open Import.log from the Documents directory (for hosted files)"
    )]
    use_docs_dir: bool,

    #[arg(long, help = "Don't watch for changes, just print once")]
    no_watch: bool,

    #[arg(long, help = "Don't print color")]
    no_color: bool,

    #[arg(long, short, help = "Only print errors")]
    errors_only: bool,
    // how should filter be passed in? what if we want multiple filters?
}

trait Line {
    fn get_colorized(&self) -> String;
    fn get_no_color(&self) -> String;
    fn print_no_color(&self) {
        println!("{}", self.get_no_color());
    }
    fn print(&self) {
        println!("{}", self.get_colorized());
    }
}
#[derive(Debug)]
struct RegularLine {
    timestamp: String,
    filename: String,
    code: String,
    message: String,
}
impl Line for RegularLine {
    fn get_colorized(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.timestamp.green(),
            self.filename.magenta(),
            self.code.yellow(),
            self.message.blue()
        )
    }

    fn get_no_color(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.timestamp, self.filename, self.code, self.message
        )
    }
}
#[derive(Debug)]
struct ErrorLine {
    timestamp: String,
    filename: String,
    code: String,
    message: String,
}

impl Line for ErrorLine {
    fn get_colorized(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.timestamp.bright_white().on_red(),
            // self.filename.bright_red(),
            self.filename.bright_white().on_red(),
            self.code.bright_white().on_red(),
            // self.message.red()
            self.message
        )
    }

    fn get_no_color(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.timestamp, self.filename, self.code, self.message
        )
    }
}

#[derive(Debug)]
struct HeaderLine {
    message: String,
}
impl Line for HeaderLine {
    fn get_no_color(&self) -> String {
        format!("{}", self.message)
    }
    fn get_colorized(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            "Timestamp".green().underline(),
            "Filename".magenta().underline(),
            "Error".yellow().underline(),
            "Message".blue().underline(),
        )
    }
}

fn is_header(line: &str) -> bool {
    line.ends_with("Timestamp\tFilename\tError\tMessage")
}

#[derive(Debug)]
enum LineType {
    Regular(RegularLine),
    Error(ErrorLine),
    Header(HeaderLine),
    Other(String),
}
impl LineType {
    fn is_header(&self) -> bool {
        match self {
            LineType::Header(_) => true,
            _ => false,
        }
    }
    fn is_error(&self) -> bool {
        match self {
            LineType::Error(_) => true,
            _ => false,
        }
    }
}
impl Line for LineType {
    fn get_no_color(&self) -> String {
        match self {
            LineType::Regular(line) => line.get_no_color(),
            LineType::Error(line) => line.get_no_color(),
            LineType::Header(line) => line.get_no_color(),
            LineType::Other(line) => line.to_string(),
        }
    }
    fn get_colorized(&self) -> String {
        match self {
            LineType::Regular(line) => line.get_colorized(),
            LineType::Error(line) => line.get_colorized(),
            LineType::Header(line) => line.get_colorized(),
            LineType::Other(line) => line.to_string(),
        }
    }
}

fn parse_line(line: &str) -> LineType {
    let v = line.splitn(4, '\t').collect::<Vec<&str>>();
    let timestamp = v.get(0).unwrap_or(&"").to_string();

    // TODO: use an enum to represent the different line types
    if is_timestamp(&timestamp) {
        let filename = v.get(1).unwrap_or(&"").to_string();
        let code = v.get(2).unwrap_or(&"").to_string();
        let message = v.get(3).unwrap_or(&"").to_string();
        if code == "0" {
            return LineType::Regular(RegularLine {
                timestamp,
                filename,
                code,
                message,
            });
        } else {
            let mut message = message;
            replace_trailing_cr_with_crlf(&mut message);
            return LineType::Error(ErrorLine {
                timestamp,
                filename,
                code,
                message,
            });
        }
    } else if is_header(&line) {
        return LineType::Header(HeaderLine {
            message: line.to_string(),
        });
    } else {
        return LineType::Other(line.to_string());
    }
}

enum PathType {
    CustomPath(PathBuf),
    CurrentDir(PathBuf),
    DocsDir(PathBuf),
}
impl PathType {
    fn message(&self) -> String {
        match self {
            PathType::CurrentDir(path) => format!("using current directory: {}", path.display()),
            PathType::DocsDir(path) => format!("using documents directory: {}", path.display()),
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
    let path = config.path.as_deref();
    match path {
        Some(path) => Ok(PathType::CustomPath(PathBuf::from(path))),
        None => {
            if config.use_docs_dir {
                let mut path = dirs::document_dir().ok_or("couldn't find documents directory")?;
                path.push("Import.log");
                if path.exists() {
                    return Ok(PathType::DocsDir(path));
                }
            } else {
                let mut path = env::current_dir().or(Err("couldn't find current directory"))?;
                path.push("Import.log");
                if path.exists() {
                    return Ok(PathType::CurrentDir(path));
                }
            }
            Err(
                "couldn't find Import.log in the documents directory. Use --help for more info"
                    .into(),
            )
        }
    }
}
pub fn run() -> CustomResult {
    let config = Config::parse();
    let path_wrapper = get_path(&config)?;
    let path = path_wrapper.path();
    let msg = path_wrapper.message();
    if !msg.is_empty() {
        let msg = if config.no_color {
            msg
        } else {
            msg.green().bold().underline().to_string()
        };
        println!("{}", msg);
    }

    let handle_line = |line: &str| {
        let line = parse_line(line);
        if config.errors_only && !line.is_error() && !line.is_header() {
            return;
        };
        if config.no_color {
            line.print_no_color()
        } else {
            line.print()
        }
    };

    let file = File::open(&path).map_err(|e| format!("couldn't open '{:?}', {}", path, e))?;
    let mut reader = io::BufReader::new(&file);
    let mut buf = String::new();

    reader.read_to_string(&mut buf).unwrap();
    buf.lines().for_each(handle_line);

    if config.no_watch {
        return Ok(());
    }

    let mut pos = buf.len() as u64;

    let (tx, rx) = mpsc::channel();
    let mut watcher = watcher(tx, Duration::from_millis(100)).unwrap();
    watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Write(_)) => {
                reader.seek(SeekFrom::Start(pos)).unwrap();
                pos = file.metadata().unwrap().len();

                buf.clear();
                reader.read_to_string(&mut buf).unwrap();
                buf.lines().for_each(handle_line);
            }
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error: {:?}", err);
                std::process::exit(1);
            }
        }
    }
}
