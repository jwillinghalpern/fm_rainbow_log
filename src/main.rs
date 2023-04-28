use clap::Parser;
use colored::Colorize;
// TODO: migrate notify to v5?
use iso8601::parsers::parse_datetime;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::mpsc;
use std::time::Duration;

#[derive(Parser, Debug)]
pub struct Config {
    #[arg(help = "The file to watch, should probably be path/to/Import.log")]
    file: String,
}

fn replace_trailing_cr_with_crlf(buf: &mut String) {
    let mut prev = 0;
    let mut new_buf = String::new();
    // I ended up having to use buf.chars() instead of buf.bytes() to preserve smart quotes, ugh
    for c in buf.chars() {
        let byte = c as u8;
        if prev == 13 && byte != 10 {
            new_buf.push(10 as char);
        }
        new_buf.push(c);
        prev = byte;
    }
    *buf = new_buf;
}

fn is_timestamp(s: &str) -> bool {
    let s = s.replace(" ", "T");
    parse_datetime(s.as_bytes()).is_ok()
}

fn print_line(line: &str) {
    // println!("{}", line);

    let v = line.splitn(4, '\t').collect::<Vec<&str>>();
    let timestamp = *v.get(0).unwrap_or(&"");
    let filename = *v.get(1).unwrap_or(&"");
    let code = *v.get(2).unwrap_or(&"");
    let message = *v.get(3).unwrap_or(&"");

    let colored_line = if !is_timestamp(timestamp) {
        // capture extra lines after an error, like when a bad multiline calc is printed, and print them red too
        // this logic assumes that the timestamp is always the first column in non-error lines
        format!("{}", line.red())
    } else if code != "0" {
        // color errors differently
        format!(
            "{}\t{}\t{}\t{}",
            timestamp.bright_white().on_red(),
            filename.bright_red(),
            code.bright_white().on_red(),
            message.red()
        )
    } else {
        // regular log msg
        format!(
            "{}\t{}\t{}\t{}",
            timestamp.green(),
            filename.magenta(),
            code.yellow(),
            message.blue()
        )
    };

    println!("{}", colored_line);
}

fn main() {
    let config = Config::parse();
    let path = config.file;

    let (tx, rx) = mpsc::channel();
    let mut watcher = watcher(tx, Duration::from_millis(100)).unwrap();
    watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

    let file = File::open(&path).unwrap();
    let mut reader = std::io::BufReader::new(&file);
    let mut buf = String::new();
    reader.read_to_string(&mut buf).unwrap();
    replace_trailing_cr_with_crlf(&mut buf);
    let mut pos = buf.len() as u64;
    buf.lines().for_each(|line| print_line(line));

    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Write(_)) => {
                // let file = File::open(&path).unwrap();
                // let mut reader = std::io::BufReader::new(&file);
                // // only read new data since last read
                reader.seek(SeekFrom::Start(pos)).unwrap();

                // move pos to end
                pos = file.metadata().unwrap().len();

                buf.clear();
                reader.read_to_string(&mut buf).unwrap();
                replace_trailing_cr_with_crlf(&mut buf);
                buf.lines().for_each(|line| print_line(line));
            }
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error: {:?}", err);
                std::process::exit(1);
            }
        }
    }
}
