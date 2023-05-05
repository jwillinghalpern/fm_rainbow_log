pub fn beep(path: &str, volume: f32) {
    let vol = if volume > 1.0 || volume < 0.0 {
        0.5
    } else {
        volume
    };
    let path = if path.is_empty() {
        "/System/Library/Sounds/Tink.aiff"
    } else {
        path
    };
    let _ = std::process::Command::new("afplay")
        .arg("-v")
        .arg(vol.to_string())
        .arg(path)
        .spawn();
}
