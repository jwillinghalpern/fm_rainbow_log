use notify_rust::Notification;
use std::sync::mpsc::Receiver;
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

fn create_notification(error_count: usize, warning_count: usize) -> Notification {
    let summary = if error_count > 0 {
        "❌ fmrl Errors 🌈"
    } else if warning_count > 0 {
        "⚠️ fmrl Warnings 🌈"
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
    Notification::new().summary(summary).body(&body).finalize()
}

fn process_messages(
    logs_rx: Receiver<NotificationType>,
    notification_sender: &dyn Fn(Notification),
) {
    println!();
    let debounce_interval = Duration::from_millis(500);
    let mut last_processed_time = Instant::now();
    let mut warning_count = 0;
    let mut error_count = 0;
    loop {
        let elapsed_time = last_processed_time.elapsed();
        if elapsed_time >= debounce_interval {
            if warning_count > 0 || error_count > 0 {
                let notification = create_notification(error_count, warning_count);
                notification_sender(notification);
                warning_count = 0;
                error_count = 0;
            }
            last_processed_time = Instant::now();
        } else if let Ok(msg) = logs_rx.recv_timeout(debounce_interval - elapsed_time) {
            match msg {
                NotificationType::Error => error_count += 1,
                NotificationType::Warning => warning_count += 1,
            }
            // Reset timer in case multiple items are pasted in quick succession
            //   or the log is backed up. This will to group more messages together.
            //   We could even add some time like `+ Duration::from_millis(250)` to wait longer.
            last_processed_time = Instant::now();
        }
    }
}

pub(crate) fn listen<F>(notif_rx: Receiver<NotificationType>, notification_sender: F)
where
    F: Fn(Notification) + Send + 'static,
{
    thread::spawn(move || {
        process_messages(notif_rx, &notification_sender);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_s() {
        assert_eq!(get_s(0), "s");
        assert_eq!(get_s(1), "");
        assert_eq!(get_s(2), "s");
    }

    #[test]
    fn test_create_notification() {
        let n = create_notification(0, 0);
        assert_eq!(n.summary, "");
        assert_eq!(n.body, "");
        let n = create_notification(1, 0);
        assert_eq!(n.summary, "🌈❌ fmrl Errors");
        assert_eq!(n.body, "1 error");
        let n = create_notification(0, 1);
        assert_eq!(n.summary, "🌈⚠️ fmrl Warnings");
        assert_eq!(n.body, "1 warning");
        let n = create_notification(1, 1);
        assert_eq!(n.summary, "🌈❌ fmrl Errors");
        assert_eq!(n.body, "1 error and 1 warning");
        let n = create_notification(2, 3);
        assert_eq!(n.summary, "🌈❌ fmrl Errors");
        assert_eq!(n.body, "2 errors and 3 warnings");
    }

    #[test]
    fn test_process_messages() {
        use std::sync::mpsc;
        let (msg_tx, msg_rx) = mpsc::channel();
        let (desktop_notifs_tx, desktop_notifs_rx) = mpsc::channel();

        // mock_sender used so we don't actually send desktop notifications while testing
        let mock_sender = move |n| desktop_notifs_tx.send(n).unwrap();
        // we still must listen for and process messages in a separate thread or else we'll block
        thread::spawn(move || {
            process_messages(msg_rx, &mock_sender);
        });

        let short_gap = Duration::from_millis(50);
        let long_gap = Duration::from_millis(550);

        // 1
        msg_tx.send(NotificationType::Error).unwrap();
        msg_tx.send(NotificationType::Warning).unwrap();
        msg_tx.send(NotificationType::Warning).unwrap();
        std::thread::sleep(short_gap); // msgs before and after should still be batched for short delay
        msg_tx.send(NotificationType::Error).unwrap();
        msg_tx.send(NotificationType::Error).unwrap();
        std::thread::sleep(short_gap); // msgs before and after should still be batched for short delay
        msg_tx.send(NotificationType::Warning).unwrap();
        msg_tx.send(NotificationType::Warning).unwrap();
        // 2
        std::thread::sleep(long_gap);
        msg_tx.send(NotificationType::Warning).unwrap();
        // 3
        std::thread::sleep(long_gap);
        msg_tx.send(NotificationType::Error).unwrap();
        msg_tx.send(NotificationType::Error).unwrap();

        // 1
        let actual = desktop_notifs_rx.recv().unwrap();
        let expected = create_notification(3, 4);
        assert_eq!(actual.summary, expected.summary);
        assert_eq!(actual.body, expected.body);
        // 2
        let actual = desktop_notifs_rx.recv().unwrap();
        let expected = create_notification(0, 1);
        assert_eq!(actual.summary, expected.summary);
        assert_eq!(actual.body, expected.body);
        // 3
        let actual = desktop_notifs_rx.recv().unwrap();
        let expected = create_notification(2, 0);
        assert_eq!(actual.summary, expected.summary);
        assert_eq!(actual.body, expected.body);
    }
}
