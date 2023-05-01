use colored::Colorize;
use serde::Deserialize;
use std::{fs::File, io::Read};

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
            "Loaded custom config from".bright_blue().underline(),
            config_path.to_string_lossy().bright_blue().underline()
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
