use colored::Colorize;
// TODO: migrate notify to v5?
use iso8601::parsers::parse_datetime;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::sync::mpsc;
use std::time::Duration;

fn is_timestamp(s: &str) -> bool {
    let s = s.replace(" ", "T");
    parse_datetime(s.as_bytes()).is_ok()
}
fn print_line(line: &str) {
    let v = line.splitn(4, '\t').collect::<Vec<&str>>();
    let timestamp = *v.get(0).unwrap_or(&"");
    let filename = *v.get(1).unwrap_or(&"");
    let code = *v.get(2).unwrap_or(&"");
    let message = *v.get(3).unwrap_or(&"");

    let colored_line = if !is_timestamp(timestamp) {
        println!("found non-timestamp line");
        format!("{}", line.red())
    } else if code != "0" {
        format!(
            "> {}\t{}\t{}\t{}",
            timestamp.bright_red(),
            filename.bright_red(),
            code.bright_white().on_red(),
            message.red()
        )
    } else {
        format!(
            "{}\t{}\t{}\t{}",
            timestamp.green(),
            filename.magenta(),
            code.yellow(),
            message.blue()
        )
        // "".to_string()
    };

    println!("{}", colored_line);
}

fn main() {
    // let s = "2023-04-27 20:50:10.648 -0700";
    // println!("{}", is_timestamp(s));
    // std::process::exit(1);

    // TODO: use clap crate to specify input and help
    let path = std::env::args().nth(1).unwrap();

    let (tx, rx) = mpsc::channel();
    let mut watcher = watcher(tx, Duration::from_millis(100)).unwrap();
    watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

    let mut contents = fs::read_to_string(&path).unwrap();
    let mut pos = contents.len() as u64;

    // TODO: print these in color
    // TODO: consider only printing the last few lines
    // print!("{}", contents);
    // contents.lines().for_each(|line| print_line(line));
    contents.lines().for_each(|line| println!("{}", line));

    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Write(_)) => {
                let mut f = File::open(&path).unwrap();
                f.seek(SeekFrom::Start(pos)).unwrap();

                pos = f.metadata().unwrap().len();

                contents.clear();
                f.read_to_string(&mut contents).unwrap();
                contents
                    .lines()
                    .into_iter()
                    .for_each(|line| print_line(line));
            }
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error: {:?}", err);
                std::process::exit(1);
            }
        }
    }
}
