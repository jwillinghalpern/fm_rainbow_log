mod config_file;
mod notifications;
mod utils;

use crate::config_file::{get_config, ConfigColor};
use crate::notifications::NotificationType;
use crate::utils::{is_timestamp, replace_trailing_cr_with_crlf};
use clap::{Command, CommandFactory, Parser, ValueHint};
use clap_complete::{generate, Generator, Shell};
use colored::{ColoredString, Colorize};
use config_file::Config;
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

    #[arg(long, short, help = "Print a separator between each import operation")]
    separator: bool,

    #[arg(long, help = "Show desktop notifications on errors and warnings")]
    notifications: bool,

    #[arg(
        long = "config",
        short = 'c',
        help = "Path to config file for customizing colors",
        value_name = "PATH"
    )]
    config_path: Option<String>,
    // how should filter be passed in? what if we want multiple filters?
    //   - maybe some basic filters and a regex option?
    #[arg(
        long,
        help = "Create log file if missing. This happens automatically when using the --docs-dir option."
    )]
    create: bool,

    #[arg(long, help = "generate completion script")]
    completion: Option<Shell>,
}

struct ImportLogLine {
    timestamp: String,
    filename: String,
    code: String,
    message: String,
}
impl ImportLogLine {
    fn contains_warning_text(&self) -> bool {
        if self.is_error() {
            return false;
        }
        let warning_checks = vec![
            // ————————————————————————————————————————————————
            // ————————————————————————————————————————————————
            |msg: &str| msg.ends_with("already exists."), // en
            |msg: &str| msg.ends_with("bereits existiert."), // de
            |msg: &str| msg.contains("ya existe"),        // es
            |msg: &str| msg.ends_with("existe déjà."),    // fr
            |msg: &str| msg.contains("esiste già") || msg.ends_with("già esistente."), // it
            |msg: &str| msg.contains("としてインポートされました。"), // ja
            |msg: &str| {
                msg.contains("이미 존재합니다")
                    || msg.contains("인 값 목록이 이미 존재함).")
                    || msg.contains("은(는) 이미 존재합니다).")
            }, // ko
            |msg: &str| msg.ends_with("bestaat."),        // nl
            |msg: &str| msg.contains("já existe"),        // pt
            |msg: &str| msg.ends_with("redan finns."),    // sv
            |msg: &str| msg.contains("名为"),             // zh
            // ————————————————————————————————————————————————
            // ————————————————————————————————————————————————
            |msg: &str| msg.ends_with("created and imported automatically."), // en
            |msg: &str| msg.ends_with("automatisch erstellt und importiert."), // de
            |msg: &str| msg.ends_with("creada e importada automáticamente."), // es
            |msg: &str| msg.ends_with("créée et importée automatiquement."),  // fr
            |msg: &str| msg.ends_with("creato e importato automaticamente."), // it
            |msg: &str| msg.contains("自動的に作成およびインポートされたファイル参照"), // ja
            |msg: &str| msg.contains("자동으로 생성되고 가져왔습니다."),      // ko
            |msg: &str| msg.ends_with("is automatisch gemaakt en geïmporteerd."), // nl
            |msg: &str| msg.ends_with("criada e importada automaticamente."), // pt
            |msg: &str| msg.ends_with("skapades och importerades automatiskt."), // sv
            |msg: &str| msg.contains("自动创建并导入丢失的文件参考"),         // zh
            // ————————————————————————————————————————————————
            // ————————————————————————————————————————————————
            |msg: &str| msg.ends_with("used instead since it refers to the same file."), // en
            |msg: &str| {
                msg.ends_with("stattdessen verwendet, da er sich auf die gleiche Datei bezieht.")
                // de
            },
            |msg: &str| msg.ends_with("ya que se refiere al mismo archivo."), // es
            |msg: &str| {
                msg.ends_with("utilisée en remplacement, car elle fait référence au même fichier.")
                // fr
            },
            |msg: &str| msg.ends_with("perché si riferisce allo stesso file."), // it
            |msg: &str| msg.contains("同じファイルを参照しているため、ファイル参照"), // ja
            |msg: &str| msg.contains("같은 파일을 참조하므로 대신 파일 참조"),  // ko
            |msg: &str| {
                msg.ends_with("worden gebruikt omdat deze naar hetzelfde bestand verwijst.")
            }, // nl
            |msg: &str| msg.ends_with("foi usada, pois faz referência ao mesmo arquivo."), // pt
            |msg: &str| msg.ends_with("används i stället, eftersom den hänvisar till samma fil."), // sv
            |msg: &str| msg.contains("因为参考同一文件，所以使用文件参考"), // zh
        ];
        warning_checks.iter().any(|check| check(&self.message))
    }
    fn is_operation_start(&self) -> bool {
        !self.is_error() && self.message.ends_with(" started")
    }
    fn is_error(&self) -> bool {
        self.code != "0"
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
    let headers = vec![
        "Timestamp\tFilename\tError\tMessage",             // en
        "Zeitstempel	Dateiname	Fehler	Meldung",               // de
        "Fecha y hora	Nombre de archivo	Error	Mensaje",       // es
        "Horodatage	NomFichier	Erreur	Message",               // fr
        "Indicatore data e ora	Nomefile	Errore	Messaggio",    // it
        "タイムスタンプ	ファイル名	エラー	メッセージ",        // ja
        "타임 스탬프	파일 이름	오류	메시지",                  // ko
        "Tijdstempel	Bestandsnaam	Fout	Bericht",              // nl
        "Carimbo de data/hora	Nome do arquivo	Erro	Mensagem", // pt
        "Tidsstämpel	Filnamn	Fel	Meddelande",                 // sv
        "时间戳	文件名	错误	信息",                            // zh
    ];
    headers.iter().any(|h| line.ends_with(h))
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
    // check timestamp before header because it's much more common
    let found_timestamp = is_timestamp(&timestamp);
    let found_header = !found_timestamp && is_header(line);
    if !found_timestamp && !found_header {
        return LineType::Other(line.to_string());
    }

    let filename = v.get(1).unwrap_or(&"").to_string();
    let code = v.get(2).unwrap_or(&"").to_string();
    let message = v.get(3).unwrap_or(&"").to_string();
    let mut line = ImportLogLine {
        timestamp,
        filename,
        code,
        message,
    };
    if found_header {
        LineType::Header(line)
    } else if line.is_error() {
        replace_trailing_cr_with_crlf(&mut line.message);
        LineType::Error(line)
    } else if line.contains_warning_text() {
        LineType::Warning(line)
    } else {
        LineType::Success(line)
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
    fn print_message(&self, no_color: bool) {
        let msg = self.message();
        if msg.is_empty() {
            return;
        }
        if no_color {
            println!("{}", msg);
        } else {
            println!("{}", msg.green().bold().underline());
        };
    }
    fn path(&self) -> &PathBuf {
        match self {
            PathType::CustomPath(path) => path,
            PathType::CurrentDir(path) => path,
            PathType::DocsDir(path) => path,
        }
    }
}

fn create_file_if_missing(path: &PathBuf, force: bool) -> CustomResult<()> {
    if !path.exists() {
        if force {
            File::create(path)
                .map_err(|_| format!("couldn't create Import.log at {}.", path.display()))?;
        } else {
            return Err(format!("couldn't find Import.log in this location. Use the --create flag to create it automatically. {}", path.display()).into());
        }
    }
    Ok(())
}

fn get_path_type(args: &Args) -> CustomResult<PathType> {
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
            let pathbuf = dirs::document_dir()
                .ok_or("couldn't find documents directory")?
                .join("Import.log");
            Ok(PathType::DocsDir(pathbuf))
        }
        _ => {
            let pathbuf = env::current_dir()
                .map_err(|_| "couldn't find current directory")?
                .join("Import.log");
            Ok(PathType::CurrentDir(pathbuf))
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

fn update_args_from_config(args: &mut Args, config: &Config) {
    if config.errors_only {
        args.errors_only = true;
    }
    if config.warnings_only {
        args.warnings_only = true;
    }
    if config.show_separator {
        args.separator = true;
    }
    if config.use_documents_directory && args.path.is_none() && args.path_unnamed.is_none() {
        args.use_docs_dir = true;
    }
}

fn generate_completion_script<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

pub fn run() -> CustomResult {
    #[cfg(target_os = "windows")]
    colored::control::set_virtual_terminal(true).unwrap();

    let mut args = Args::parse();
    if let Some(gen) = args.completion {
        // let mut gen = Generator::new(generator);
        let mut cmd = Args::command();
        generate_completion_script(gen, &mut cmd);
        return Ok(());
    }

    let config = get_config(args.config_path.as_deref())?;
    update_args_from_config(&mut args, &config);

    let path_type = get_path_type(&args)?;
    let path = path_type.path();

    // NOTE: docs dir is the only folder where we force create the file by default. The others require the --create flag.
    create_file_if_missing(
        path,
        args.create || matches!(path_type, PathType::DocsDir(_)),
    )?;
    path_type.print_message(args.no_color);

    // get colorizer for each field:
    let timestamp_colorizer = get_default_colorizer(config.colors.timestamp, "cyan".to_string());
    let filename_colorizer = get_default_colorizer(config.colors.filename, "green".to_string());
    let error_colorizer = get_default_colorizer(config.colors.error, "bright magenta".to_string());
    let message_colorizer = get_default_colorizer(config.colors.message, "bright blue".to_string());

    // create a channel whether we send notifications or not, because the handle_line closure needs one, even if it doesn't do anything.
    let (notif_tx, notif_rx) = mpsc::channel();
    if args.notifications {
        notifications::listen(notif_rx);
    }

    // closure/fn to handle each line
    let handle_line = |line: &str, send_notif: bool| {
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
                    // warning_notification();
                    if send_notif {
                        notif_tx.send(NotificationType::Error).unwrap();
                    }
                }
                LineType::Warning(line) => {
                    println!(
                        "{} {} {} {}",
                        line.timestamp.black().on_yellow(),
                        line.filename.black().on_yellow(),
                        line.code.black().on_yellow(),
                        line.message
                    );
                    // warning_notification();
                    if send_notif {
                        notif_tx.send(NotificationType::Warning).unwrap();
                    }
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
                LineType::Other(line) => println!("{}", line),
            }
        }
    };

    let file = File::open(path).map_err(|e| format!("couldn't open '{:?}', {}", path, e))?;
    let mut reader = io::BufReader::new(&file);
    let mut buf = String::new();

    // read the initial file content
    reader.read_to_string(&mut buf).unwrap();
    // don't send_notif for intitial file content. It might be a ton of old errors and warnings
    buf.lines().for_each(|line| handle_line(line, false));
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
                buf.lines()
                    .for_each(|line| handle_line(line, args.notifications));
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
        // TODO: we should create files containing a list of headers and non-headers
        assert!(is_header(
            "anythinghere-- Timestamp\tFilename\tError\tMessage"
        ));
        let line = "hello world";
        assert!(!is_header(line));
        assert!(is_header("lkjflkjfljf - 타임 스탬프	파일 이름	오류	메시지"))
    }
}
