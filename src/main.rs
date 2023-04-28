use colored::Colorize;
// TODO: migrate notify to v5?
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::sync::mpsc;
use std::time::Duration;

fn print_line(line: &str) {
    // split line by \t and colorize each column differently using the colored crate:
    // https://docs.rs/colored/2.0.0/colored/

    let columns = line.split("\t");
    let mut colored_line = String::new();
    let mut color = 0;
    // TODO: highlight errors in red and bold
    // TODO: use colored crate
    let mut columns_iter = columns.enumerate();
    while let Some((_idx, column)) = columns_iter.next() {
        colored_line.push_str(&format!(
            "{}{}\t",
            column,
            match color {
                // 0 => "\x1b[0;31m",
                0 => "\x1b[0;32m",
                1 => "\x1b[0;33m",
                2 => "\x1b[0;34m",
                3 => "\x1b[0;35m",
                4 => "\x1b[0;36m",
                5 => "\x1b[0;37m",
                _ => "\x1b[0m",
            }
        ));
        color += 1;
        if color > 6 {
            color = 0;
        }
    }
    colored_line.push_str("\x1b[0m");
    println!("{}", colored_line);
}

fn main() {
    let path = std::env::args().nth(1).unwrap();

    let (tx, rx) = mpsc::channel();
    let mut watcher = watcher(tx, Duration::from_millis(100)).unwrap();
    watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

    let mut contents = fs::read_to_string(&path).unwrap();
    let mut pos = contents.len() as u64;

    // TODO: print these in color
    // TODO: consider only printing the last few lines
    print!("{}", contents);

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
