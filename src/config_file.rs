use crate::Args;
use colored::Colorize;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub(crate) struct ConfigColor {
    pub(crate) foreground: String,
    pub(crate) background: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub(crate) struct ConfigColorFields {
    pub(crate) timestamp: ConfigColor,
    pub(crate) filename: ConfigColor,
    pub(crate) error: ConfigColor,
    pub(crate) message: ConfigColor,
}
#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) show_separator: bool,
    pub(crate) use_documents_directory: bool,
    pub(crate) errors_only: bool,
    pub(crate) warnings_only: bool,
    pub(crate) show_notifications: bool,
    pub(crate) beep: bool,
    pub(crate) beep_volume: f32,
    pub(crate) beep_path: String,
    pub(crate) colors: ConfigColorFields,
}

fn get_default_config_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    Ok(dirs::config_dir()
        .ok_or("couldn't find config directory")?
        .join("fm_rainbow_log")
        .join("config.json"))
}
pub(crate) fn get_config(config_path: Option<&str>) -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = if let Some(config_path) = config_path {
        // custom path provided, this should error if the file doesn't exist
        let config_path = std::path::PathBuf::from(config_path);
        if !config_path.exists() {
            return Err(format!("config file doesn't exist: {:?}", config_path).into());
        }
        config_path
    } else {
        // checking for optional default config file. This should NOT error if the file doesn't exist
        let config_path = get_default_config_path()?;
        if !config_path.exists() {
            return Ok(Config::default());
        };
        println!(
            "{} {}",
            "Loaded custom config from".bright_blue().underline().bold(),
            config_path
                .to_string_lossy()
                .bright_blue()
                .underline()
                .bold()
        );
        config_path
    };

    let mut buf = String::new();
    File::open(config_path)
        .map_err(|e| format!("couln't open config file: {}", e))?
        .read_to_string(&mut buf)
        .map_err(|e| format!("couldn't read config file: {}", e))?;

    let config =
        serde_json::from_str(&buf).map_err(|e| format!("couldn't parse config file: {}", e))?;

    Ok(config)
}

pub(crate) fn update_args_from_config(args: &mut Args, config: &Config) {
    if config.errors_only {
        args.errors_only = true;
    }
    if config.warnings_only {
        args.warnings_only = true;
    }
    if config.show_separator {
        args.separator = true;
    }
    if config.show_notifications {
        args.notifications = true;
    }
    if config.use_documents_directory && args.path.is_none() && args.path_unnamed.is_none() {
        args.use_docs_dir = true;
    }
    if config.beep {
        args.beep = true;
    }
    if !config.beep_path.is_empty() {
        args.beep_path = config.beep_path.clone();
    }
    if config.beep_volume > 0.0 {
        args.beep_volume = config.beep_volume;
    }
}
