use crate::ImportLogLine;
use serde::{Deserialize, Deserializer};

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

#[derive(Deserialize, Debug, Clone, Default, PartialEq)]
pub(crate) struct ErrorRule {
    #[serde(deserialize_with = "deserialize_error_code", default)]
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

/// converts a number or string to Option<String>
fn deserialize_error_code<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;

    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    let string_value = match value {
        Value::Number(num) => Some(num.to_string()),
        Value::String(s) if s.is_empty() => None,
        Value::String(s) if !contains_only_digits(&s) => {
            return Err(serde::de::Error::custom(format!(
                "Expected a number or a string containing only digits, got: {s}"
            )));
        }
        Value::String(s) => Some(s),
        _ => return Err(serde::de::Error::custom("Expected a number or a string")),
    };

    Ok(string_value)
}

fn contains_only_digits(input: &str) -> bool {
    input.chars().all(|c| c.is_digit(10))
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

    /// check if only the action field is set on this rule. If so, then the rule has no match logic so will never be used and can be ignored.
    fn no_match_logic(&self) -> bool {
        // set the non-optional fields to the same on boths sides of the comparison, then compare against the default ErrorRule. default Options are None, so if they match, we can assume all optional fields are None
        let action = ErrorRuleAction::default();
        let default = ErrorRule {
            action,
            ..ErrorRule::default()
        };

        let this_rule = ErrorRule {
            action,
            ..self.clone()
        };

        default == this_rule
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

pub(crate) fn remove_no_match_rules(rules: &mut Vec<ErrorRule>) {
    rules.retain(|rule| !rule.no_match_logic());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_error_rule() {
        let json = r#"{"error_code": "123", "message_contains": "abc", "action": "quiet"}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.error_code, Some("123".to_string()));
        assert_eq!(res.message_contains, Some("abc".to_string()));
        assert_eq!(res.action, ErrorRuleAction::Quiet);

        let json = r#"{"message_contains": "abc", "action": "ignore"}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.error_code, None);
        assert_eq!(res.message_contains, Some("abc".to_string()));
        assert_eq!(res.action, ErrorRuleAction::Ignore);

        let json = r#"{"error_code": "123", "action": "ignore"}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.error_code, Some("123".to_string()));
        assert_eq!(res.message_contains, None);
        assert_eq!(res.action, ErrorRuleAction::Ignore);

        let json = r#"{"action": "ignore"}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.error_code, None);
        assert_eq!(res.message_contains, None);
        assert_eq!(res.action, ErrorRuleAction::Ignore);

        // only action:
        let json = r#"{"action": "ignore"}"#;
        serde_json::from_str::<ErrorRule>(json).unwrap();

        // add more tests when we add more properties like message_starts_with etc.
        let json = r#"{
            "action": "quiet",
            "error_code": "123",
            "message_contains": "abc",
            "message_starts_with": "abc",
            "message_ends_with": "def",
            "location_contains": "ghi",
            "location_starts_with": "jkl",
            "location_ends_with": "mno"
        }"#;
        serde_json::from_str::<ErrorRule>(json).unwrap();
    }

    #[test]
    fn error_code_can_be_num_or_string() {
        let json = r#"{"action": "quiet", "error_code": "123"}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.error_code, Some("123".to_string()));

        let json = r#"{"action": "quiet", "error_code": 123}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.error_code, Some("123".to_string()));
    }

    #[test]
    fn empty_error_code_deserializes_to_none() {
        let json = r#"{"action": "quiet", "error_code": ""}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.error_code, None);
    }

    #[test]
    fn error_code_deserializer_fails_on_non_digits() {
        let json = r#"{"action": "quiet", "error_code": "LETTERS"}"#;
        let res: Result<ErrorRule, _> = serde_json::from_str(json);
        assert!(res.is_err());

        let json = r#"{"action": "quiet", "error_code": "123LETTERS"}"#;
        let res: Result<ErrorRule, _> = serde_json::from_str(json);
        assert!(res.is_err());

        // spaces are not allowed
        let json = r#"{"action": "quiet", "error_code": "123 234 345"}"#;
        let res: Result<ErrorRule, _> = serde_json::from_str(json);
        assert!(res.is_err());
    }

    #[test]
    fn deserialize_error_rule_action() {
        let json = r#"{"action": "quiet"}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.action, ErrorRuleAction::Quiet);

        let json = r#"{"action": "ignore"}"#;
        let res: ErrorRule = serde_json::from_str(json).unwrap();
        assert_eq!(res.action, ErrorRuleAction::Ignore);

        let json = r#"{ "action": "INVALID_VARIANT"}"#;
        let res: std::result::Result<ErrorRule, _> = serde_json::from_str(json);
        assert!(res.is_err());
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
        // TODO: test each of the match logic fields individually
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

    #[test]
    fn apply_error_rules_works() {
        let rules = vec![
            ErrorRule {
                error_code: Some("123".to_string()),
                message_contains: Some("abc".to_string()),
                action: ErrorRuleAction::Quiet,
                ..ErrorRule::default()
            },
            ErrorRule {
                error_code: Some("456".to_string()),
                message_contains: Some("def".to_string()),
                action: ErrorRuleAction::Ignore,
                ..ErrorRule::default()
            },
        ];

        let line = ImportLogLine {
            code: "123".to_string(),
            message: "HELLO_abc_WORLD".to_string(),
            ..ImportLogLine::default()
        };
        let res = apply_error_rules(&rules, &line);
        assert_eq!(res, Some(ErrorRuleAction::Quiet));

        let line = ImportLogLine {
            code: "456".to_string(),
            message: "HELLO_def_WORLD".to_string(),
            ..ImportLogLine::default()
        };
        let res = apply_error_rules(&rules, &line);
        assert_eq!(res, Some(ErrorRuleAction::Ignore));
    }

    #[test]
    fn no_match_logic_works() {
        let rule = ErrorRule::default();
        assert!(rule.no_match_logic());

        let rule = ErrorRule {
            error_code: Some("123".to_string()),
            ..ErrorRule::default()
        };
        assert!(!rule.no_match_logic());
    }
}
