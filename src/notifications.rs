use notify_rust::{Notification, Timeout};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

pub(crate) enum NotificationType {
    Error,
    Warning,
}

fn get_s(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

fn send_notification(error_count: usize, warning_count: usize) {
    let summary = if error_count > 0 {
        "🌈❌ fmrl Errors"
    } else if warning_count > 0 {
        "🌈⚠️ fmlr Warnings"
    } else {
        ""
    };
    let mut body = String::new();
    if error_count > 0 {
        let s = get_s(error_count);
        body.push_str(format!("{error_count} error{s}").as_str());
    };
    if warning_count > 0 {
        if !body.is_empty() {
            body.push_str(" and ");
        }
        let s = get_s(warning_count);
        body.push_str(format!("{warning_count} warning{s}").as_str());
    };
    Notification::new()
        .summary(summary)
        .body(&body)
        // // icon will only work on windows. I should pick one that is likely on most computers (powershell?)
        // .icon("firefox")
        .timeout(Timeout::Milliseconds(6000)) //milliseconds
        .show()
        .unwrap();
}

pub(crate) fn process_messages(logs_rx: Receiver<NotificationType>) {
    let debounce_interval = Duration::from_millis(1000);
    let mut last_processed_time = Instant::now();
    let mut warning_count = 0;
    let mut error_count = 0;
    loop {
        let elapsed_time = last_processed_time.elapsed();
        if elapsed_time >= debounce_interval {
            if warning_count > 0 || error_count > 0 {
                send_notification(error_count, warning_count);
                warning_count = 0;
                error_count = 0;
            }
            last_processed_time = Instant::now();
        } else if let Ok(msg) = logs_rx.recv_timeout(debounce_interval - elapsed_time) {
            match msg {
                NotificationType::Error => error_count += 1,
                NotificationType::Warning => warning_count += 1,
            }
        }
    }
}

// spawn a thread to send notifications asynchronously
// TODO: only spawn notifications thread if the notifications option is set
// if args.notifications {}
pub(crate) fn listen() -> Sender<NotificationType> {
    let (notif_tx, notif_rx) = mpsc::channel();
    thread::spawn(move || {
        process_messages(notif_rx);
    });
    notif_tx
}
