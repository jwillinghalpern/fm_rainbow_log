use crate::ImportLogLine;
use serde::Deserialize;
use std::str::FromStr;

//TODO: rename file

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ErrorRuleAction {
    Quiet,
    Ignore,
}

impl Default for ErrorRuleAction {
    fn default() -> Self {
        ErrorRuleAction::Quiet
    }
}

impl FromStr for ErrorRuleAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "quiet" => Ok(ErrorRuleAction::Quiet),
            "ignore" => Ok(ErrorRuleAction::Ignore),
            _ => Err(format!("unknown error rule action: {}", s)),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
// #[derive(Deserialize)]
pub(crate) struct ErrorRule {
    // error_code is optional, but if it's empty, then any non-zero error code will have this rule applied
    error_code: Option<String>,
    // rules act like an AND query clause. All rules must match for the rule to be satisfied, this lets you get specific about the shape of an error. e.g. starts with "foo" and contains "bar" and ends with "."
    message_contains: Option<String>,
    message_starts_with: Option<String>,
    message_ends_with: Option<String>,
    location_contains: Option<String>,
    location_starts_with: Option<String>,
    location_ends_with: Option<String>,
    action: ErrorRuleAction,
}

// TODO: do i still need this with the custom parse in args?
impl FromStr for ErrorRule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res: ErrorRule =
            serde_json::from_str(s).map_err(|e| format!("error parsing error rule: {}", e))?;
        Ok(res)
    }
}

impl ErrorRule {
    /// check if ImportLogLine matches this rule
    fn get_action(&self, line: &ImportLogLine) -> Option<ErrorRuleAction> {
        // we only care about error lines (non-zero error code)
        if line.code == "0" {
            return None;
        };

        if let Some(error_code) = &self.error_code {
            if error_code != &line.code {
                return None;
            }
        }

        if let Some(message_contains) = &self.message_contains {
            if !line.message.contains(message_contains) {
                return None;
            }
        }

        if let Some(message_starts_with) = &self.message_starts_with {
            if !line.message.starts_with(message_starts_with) {
                return None;
            }
        }

        if let Some(message_ends_with) = &self.message_ends_with {
            if !line.message.ends_with(message_ends_with) {
                return None;
            }
        }

        if let Some(location_contains) = &self.location_contains {
            if !line.filename.contains(location_contains) {
                return None;
            }
        }

        if let Some(location_starts_with) = &self.location_starts_with {
            if !line.filename.starts_with(location_starts_with) {
                return None;
            }
        }

        if let Some(location_ends_with) = &self.location_ends_with {
            if !line.filename.ends_with(location_ends_with) {
                return None;
            }
        }

        // return the action if all the rules match
        Some(self.action)
    }

    // write function that checks if the only property defined is action
    pub(crate) fn no_match_logic(&self) -> bool {
        // TODO: update this so that you don't have to remember to add new fields

        self.error_code.is_none()
            && self.message_contains.is_none()
            && self.location_contains.is_none()
            && self.message_starts_with.is_none()
            && self.message_ends_with.is_none()
            && self.location_starts_with.is_none()
            && self.location_ends_with.is_none()
    }
}

pub(crate) fn apply_error_rules(
    rules: &[ErrorRule],
    line: &ImportLogLine,
) -> Option<ErrorRuleAction> {
    use ErrorRuleAction::*;
    let mut action = None;
    for rule in rules {
        match rule.get_action(line) {
            Some(Quiet) => action = Some(Quiet),
            Some(Ignore) => return Some(Ignore),
            None => continue,
        }
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_error_rule() {
        let json = r#"{"error_code": 123, "message_contains": "abc", "action": "quiet"}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.error_code, Some("123".to_string()));
        assert_eq!(res.message_contains, Some("abc".to_string()));
        assert_eq!(res.action, ErrorRuleAction::Quiet);

        let json = r#"{"message_contains": "abc", "action": "ignore"}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.error_code, None);
        assert_eq!(res.message_contains, Some("abc".to_string()));
        assert_eq!(res.action, ErrorRuleAction::Ignore);

        let json = r#"{"error_code": 123, "action": "ignore"}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.error_code, Some("123".to_string()));
        assert_eq!(res.message_contains, None);
        assert_eq!(res.action, ErrorRuleAction::Ignore);

        // TODO: IDK if this is correct
        let json = r#"{"error_code": 123, "message_contains": "abc", "message_contains": "def", "action": "quiet"}"#;
        let res: std::result::Result<ErrorRule, _> = serde_json::from_str(json);
        assert!(res.is_err());

        let json = r#"{"action": "ignore"}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.error_code, None);
        assert_eq!(res.message_contains, None);
        assert_eq!(res.action, ErrorRuleAction::Ignore);

        // TODO: add more tests when we add more properties lik message_starts_with etc.
    }

    #[test]
    fn deserialize_error_rule_array() {
        let json = r#"[
				{"error_code": "123", "message_contains": "abc", "action": "quiet"},
				{"error_code": "456", "message_contains": "def", "action": "ignore"}
			]"#;
        let res: Vec<ErrorRule> = serde_json::from_str(json).unwrap();
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn get_action_works() {
        let rule = ErrorRule {
            error_code: Some("123".to_string()),
            message_contains: Some("abc".to_string()),
            action: ErrorRuleAction::Quiet,
            location_contains: None,
            ..ErrorRule::default()
        };

        let line = ImportLogLine {
            code: "123".to_string(),
            message: "HELLO_abc_WORLD".to_string(),
            ..ImportLogLine::default()
        };
        assert_eq!(rule.get_action(&line), Some(ErrorRuleAction::Quiet));

        let line = ImportLogLine {
            code: "123".to_string(),
            message: "does not match".to_string(),
            ..ImportLogLine::default()
        };
        assert_eq!(rule.get_action(&line), None);

        // let rule = ErrorRule {
        //     error_code: None,
        //     message_contains: Some("abc".to_string()),
        //     action: ErrorRuleAction::Quiet,
        //     ..ErrorRule::default()
        // };
        // let line = ImportLogLine {
        //     code: "123".to_string(),
        //     message: "HELLO_abc_WORLD".to_string(),
        //     ..ImportLogLine::default()
        // };
    }

    #[test]
    fn get_action_matches_any_error_if_error_code_is_none() {
        let rule = ErrorRule {
            error_code: None,
            message_contains: Some("abc".to_string()),
            action: ErrorRuleAction::Quiet,
            ..ErrorRule::default()
        };

        let line = ImportLogLine {
            code: "123".to_string(),
            message: "HELLO_abc_WORLD".to_string(),
            ..ImportLogLine::default()
        };
        assert_eq!(rule.get_action(&line), Some(ErrorRuleAction::Quiet));

        let line = ImportLogLine {
            code: "456".to_string(),
            message: "HELLO_abc_WORLD".to_string(),
            ..ImportLogLine::default()
        };
        assert_eq!(rule.get_action(&line), Some(ErrorRuleAction::Quiet));
    }

    #[test]
    fn get_action_returns_none_for_non_error_lines() {
        let rule = ErrorRule {
            error_code: Some("123".to_string()),
            message_contains: Some("abc".to_string()),
            // location_contains: None,
            action: ErrorRuleAction::Quiet,
            ..ErrorRule::default()
        };
        let line = ImportLogLine {
            code: "0".to_string(),
            message: "HELLO_abc_WORLD".to_string(),
            ..ImportLogLine::default()
        };
        assert_eq!(rule.get_action(&line), None);
    }

    // TODO: add test other ErrorRule fields
    // TODO: test function that applies rules

    #[test]
    fn apply_error_rules_works() {
        // let line = ImportLogLine {
        //     code: "123".to_string(),
        //     message: "HELLO_abc_WORLD".to_string(),
        //     ..ImportLogLine::default()
        // };
        // TODO
    }
}
