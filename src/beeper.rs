#[cfg(target_os = "macos")]
pub fn beep(path: &str, volume: f32) {
    let vol = if !(0.0..=1.0).contains(&volume) {
        1.0
    } else {
        volume
    };
    let path = if path.is_empty() {
        "/System/Library/Sounds/Tink.aiff"
    } else {
        path
    };
    if let Ok(mut child) = std::process::Command::new("afplay")
        .arg("-v")
        .arg(vol.to_string())
        .arg(path)
        .spawn()
    {
        // previx underscore to avoid conditional compilation warning in release build.
        // _res is only used in debug mode.
        let _res = child.wait();
        #[cfg(debug_assertions)]
        if let Err(e) = _res {
            eprintln!("afplay wait error: {}", e);
        };
    } else {
        #[cfg(debug_assertions)]
        eprintln!("afplay not found");
    }
}

#[cfg(target_os = "windows")]
pub fn beep(_path: &str, _volume: f32) {}
