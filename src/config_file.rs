use crate::error_rule::{remove_no_match_rules, ErrorRule};
use crate::Args;
use colored::Colorize;
use serde::{Deserialize, Deserializer};
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
    #[serde(deserialize_with = "comma_list_deserialize")]
    pub(crate) quiet_errors: Vec<String>,
    pub(crate) error_rules: Vec<ErrorRule>,
}

fn comma_list_deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let str_sequence = String::deserialize(deserializer)?;
    Ok(str_sequence
        .split(',')
        .map(|item| item.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect())
}

pub(crate) fn get_default_config_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
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
                .display()
                .to_string()
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

    let mut config: Config =
        json5::from_str(&buf).map_err(|e| format!("couldn't parse config file: {}", e))?;

    remove_no_match_rules(&mut config.error_rules);

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
    if !config.quiet_errors.is_empty() && args.quiet_errors.is_empty() {
        args.quiet_errors = config.quiet_errors.clone();
    }
    if !config.error_rules.is_empty() && args.error_rules.is_empty() {
        args.error_rules = config.error_rules.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_errors_should_not_overwrite_non_empty() {
        let mut args = Args::default();
        args.quiet_errors = vec!["111".to_string()];
        let mut config = Config::default();
        config.quiet_errors = vec!["999".to_string()];
        update_args_from_config(&mut args, &config);
        assert_eq!(args.quiet_errors, vec!["111".to_string()]);
    }

    #[test]
    fn quiet_errors_should_overwrite_empty() {
        let mut args = Args::default();
        args.quiet_errors = vec![];
        let mut config = Config::default();
        config.quiet_errors = vec!["999".to_string()];
        update_args_from_config(&mut args, &config);
        assert_eq!(args.quiet_errors, vec!["999".to_string()]);
    }

    #[test]
    fn quiet_errors_empty_should_not_overwrite_non_empty() {
        let mut args = Args::default();
        args.quiet_errors = vec!["111".to_string()];
        let mut config = Config::default();
        config.quiet_errors = vec![];
        update_args_from_config(&mut args, &config);
        assert_eq!(args.quiet_errors, vec!["111".to_string()]);
    }

    #[test]
    fn test_comma_list_deserialize() {
        let my_struct = r#"
        {
            "my_field": "a,b,c"
        }"#;

        #[derive(Deserialize, Debug)]
        struct MyStruct {
            #[serde(deserialize_with = "comma_list_deserialize")]
            my_field: Vec<String>,
        }

        let res: MyStruct = serde_json::from_str(my_struct).unwrap();
        let my_field_expected = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(res.my_field, my_field_expected);

        // bad formatting:
        let my_struct = r#"
        {
            "my_field": "123,     234   ,,,, 234  , "
        }"#;
        let res: MyStruct = serde_json::from_str(my_struct).unwrap();
        let my_field_expected = vec!["123".to_string(), "234".to_string(), "234".to_string()];
        assert_eq!(res.my_field, my_field_expected);
    }
}
