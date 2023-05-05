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
    let _ = std::process::Command::new("afplay")
        .arg("-v")
        .arg(vol.to_string())
        .arg(path)
        .spawn();
}
