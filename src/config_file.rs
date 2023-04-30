use serde::Deserialize;
use serde_json;
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
pub(crate) fn get_config(config_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut buf = String::new();
    File::open(config_path)
        .map_err(|e| format!("couln't open config file: {}", e))?
        .read_to_string(&mut buf)
        .map_err(|e| format!("couldn't read config file: {}", e))?;
    let config: Config =
        serde_json::from_str(&buf).map_err(|e| format!("couldn't parse config file: {}", e))?;
    Ok(config)
}
