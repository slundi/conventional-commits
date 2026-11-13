//! Commit message validation.

use regex::Regex;

use crate::commit::{CommitType, ConventionalCommit};
use crate::error::{CommitValidationError, ValidationErrors};
use crate::parse::{is_breaking_footer, is_footer, parse_header};

/// Configuration for commit validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum length for the description
    pub max_description_length: usize,
    /// Whether to allow custom commit types
    pub allow_custom_types: bool,
    /// Whether to enforce lowercase description
    pub enforce_lowercase_description: bool,
    /// Whether to disallow period at end of description
    pub disallow_description_period: bool,
    /// Optional list of allowed scopes; if `None`, any scope is accepted
    pub allowed_scopes: Option<Vec<String>>,
    /// Whether to enforce lowercase scope
    pub enforce_lowercase_scope: bool,
    /// Whether a scope is required on every commit
    pub require_scope: bool,
    /// Optional regex pattern that at least one footer must match (e.g. `r"^Refs: #\d+"`)
    pub issue_pattern: Option<String>,
    /// Minimum length for the description (0 means no minimum beyond non-empty)
    pub min_description_length: usize,
    /// Optional list of allowed types; if `None`, all types are accepted
    pub allowed_types: Option<Vec<String>>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_description_length: 72,
            allow_custom_types: true,
            enforce_lowercase_description: true,
            disallow_description_period: true,
            allowed_scopes: None,
            enforce_lowercase_scope: false,
            require_scope: false,
            issue_pattern: None,
            min_description_length: 0,
            allowed_types: None,
        }
    }
}

/// Validates a commit message according to Conventional Commits specification
///
/// # Arguments
///
/// * `message` - The commit message to validate
///
/// # Returns
///
/// Returns `Ok(ConventionalCommit)` if valid, or `Err(CommitValidationError)` if invalid
///
/// # Examples
///
/// ```
/// use conventional_commits::validate_commit;
///
/// let result = validate_commit("feat: add user authentication");
/// assert!(result.is_ok());
///
/// let result = validate_commit("invalid commit message");
/// assert!(result.is_err());
/// ```
pub fn validate_commit(message: &str) -> Result<ConventionalCommit, CommitValidationError> {
    validate_commit_with_config(message, &ValidationConfig::default())
}

/// Validates a commit message with custom configuration
///
/// # Arguments
///
/// * `message` - The commit message to validate
/// * `config` - Custom validation configuration
///
/// # Returns
///
/// Returns `Ok(ConventionalCommit)` if valid, or `Err(CommitValidationError)` if invalid
pub fn validate_commit_with_config(
    message: &str,
    config: &ValidationConfig,
) -> Result<ConventionalCommit, CommitValidationError> {
    if message.trim().is_empty() {
        return Err(CommitValidationError::EmptyMessage);
    }

    let lines: Vec<&str> = message.lines().collect();
    let header = lines[0];

    // Header parsing fails fast — without a parseable header we cannot continue.
    let (type_str, scope, breaking_change, description) = parse_header(header)?;

    let mut errors: Vec<CommitValidationError> = Vec::new();

    // Validate Blank Line Rule:
    // If there is more than one line, the second line MUST be empty.
    if lines.len() > 1 && !lines[1].trim().is_empty() {
        errors.push(CommitValidationError::MissingBlankLine);
    }

    // Validate commit type
    let commit_type = CommitType::from(type_str.as_str());
    if !config.allow_custom_types
        && let CommitType::Custom(_) = commit_type
    {
        errors.push(CommitValidationError::UnknownType(type_str.clone()));
    }
    if let Some(ref allowed) = config.allowed_types
        && !allowed.contains(&type_str)
    {
        errors.push(CommitValidationError::TypeNotAllowed(
            type_str.clone(),
            allowed.join(", "),
        ));
    }

    // Validate scope (only when a scope is present)
    if let Some(ref scope_name) = scope {
        if let Some(ref allowed) = config.allowed_scopes
            && !allowed
                .iter()
                .any(|allowed_scope| allowed_scope == scope_name)
        {
            errors.push(CommitValidationError::ScopeNotAllowed(scope_name.clone()));
        }
        if config.enforce_lowercase_scope && scope_name.chars().any(char::is_uppercase) {
            errors.push(CommitValidationError::ScopeNotLowercase(scope_name.clone()));
        }
    } else if config.require_scope {
        errors.push(CommitValidationError::ScopeRequired);
    }

    // Validate description
    if config.enforce_lowercase_description
        && let Some(first) = description.chars().next()
        && !first.is_lowercase()
    {
        errors.push(CommitValidationError::DescriptionNotLowercase);
    }

    if config.disallow_description_period && description.ends_with('.') {
        errors.push(CommitValidationError::DescriptionEndsWithPeriod);
    }

    if description.len() > config.max_description_length {
        errors.push(CommitValidationError::DescriptionTooLong);
    }

    if config.min_description_length > 0 && description.len() < config.min_description_length {
        errors.push(CommitValidationError::DescriptionTooShort(
            config.min_description_length,
        ));
    }

    // Parse body and footers
    let mut body_lines: Vec<String> = Vec::new();
    let mut footer_lines: Vec<String> = Vec::new();
    let mut is_breaking = breaking_change;

    if lines.len() > 2 {
        for line in lines[2..].iter() {
            if is_breaking_footer(line) || is_footer(line) {
                footer_lines.push(line.to_string());
                if is_breaking_footer(line) {
                    is_breaking = true;
                }
            } else {
                if !footer_lines.is_empty() {
                    body_lines.append(&mut footer_lines);
                }
                body_lines.push((*line).to_string());
            }
        }
    }

    // Validate issue pattern: at least one footer must match
    if let Some(ref pattern) = config.issue_pattern {
        let issue_regex = Regex::new(pattern).map_err(|error| {
            CommitValidationError::InvalidIssuePattern(pattern.clone(), error.to_string())
        })?;
        if !footer_lines
            .iter()
            .any(|footer| issue_regex.is_match(footer))
        {
            errors.push(CommitValidationError::MissingIssueRef(pattern.clone()));
        }
    }

    if errors.len() == 1 {
        return Err(errors.remove(0));
    } else if errors.len() > 1 {
        return Err(CommitValidationError::Multiple(ValidationErrors(errors)));
    }

    let body = if body_lines.is_empty() {
        None
    } else {
        Some(body_lines.join("\n").trim().to_string())
    };

    Ok(ConventionalCommit {
        commit_type,
        scope,
        breaking_change: is_breaking,
        description,
        body,
        footers: footer_lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_commits() {
        let valid_commits = vec![
            "feat: add user authentication",
            "feat(auth): add OAuth support",
            "feat!: breaking change",
            "fix: bug\n\nBody message", // Standard compliant
        ];

        for commit in valid_commits {
            let res = validate_commit(commit);
            assert!(res.is_ok(), "Failed on: {}\nError: {:?}", commit, res.err());
        }
    }

    #[test]
    fn test_invalid_commits() {
        let invalid = vec![
            ("feat:no-space", "Missing space after colon"),
            (
                "feat: description\nNo blank line",
                "Missing blank line rule",
            ),
        ];

        for (msg, reason) in invalid {
            let res = validate_commit(msg);
            assert!(res.is_err(), "Should fail for {}: {}", reason, msg);
        }
    }

    #[test]
    fn test_empty_message() {
        assert!(matches!(
            validate_commit("   \n "),
            Err(CommitValidationError::EmptyMessage)
        ));
    }

    #[test]
    fn test_custom_type_rejected_when_disallowed() {
        let cfg = ValidationConfig {
            allow_custom_types: false,
            ..ValidationConfig::default()
        };
        assert!(matches!(
            validate_commit_with_config("wip: stuff", &cfg),
            Err(CommitValidationError::UnknownType(ref name)) if name == "wip"
        ));
        assert!(validate_commit_with_config("feat: stuff", &cfg).is_ok());
    }

    #[test]
    fn test_description_too_long() {
        let cfg = ValidationConfig {
            max_description_length: 5,
            ..ValidationConfig::default()
        };
        assert!(matches!(
            validate_commit_with_config("feat: add thing", &cfg),
            Err(CommitValidationError::DescriptionTooLong)
        ));
    }

    #[test]
    fn test_body_and_footers_split() {
        let commit =
            validate_commit("fix: bug\n\nLine one\nLine two\n\nRefs: #1\nReviewed-by: Bob")
                .unwrap();
        assert_eq!(commit.body.as_deref(), Some("Line one\nLine two"));
        assert_eq!(commit.footers, vec!["Refs: #1", "Reviewed-by: Bob"]);
    }

    #[test]
    fn test_footer_like_line_followed_by_body_is_body() {
        let commit = validate_commit("fix: bug\n\nRefs: #1\nmore body").unwrap();
        assert_eq!(commit.body.as_deref(), Some("Refs: #1\nmore body"));
        assert!(commit.footers.is_empty());
    }

    #[test]
    fn test_breaking_change_footer() {
        let msg = "feat: logic\n\nBREAKING CHANGE: this is a footer";
        let commit = validate_commit(msg).unwrap();
        assert!(
            commit.breaking_change,
            "Footer should trigger breaking_change flag"
        );
    }

    #[test]
    fn test_breaking_change_detection() {
        let commit = validate_commit("feat!: breaking change").unwrap();
        assert!(commit.breaking_change);

        let commit_with_footer =
            validate_commit("feat: new feature\n\nBREAKING CHANGE: this breaks something").unwrap();
        assert!(commit_with_footer.is_breaking_change());
    }

    // ---- scope-enum tests ----

    fn config_with_scopes(scopes: &[&str]) -> ValidationConfig {
        ValidationConfig {
            allowed_scopes: Some(scopes.iter().map(ToString::to_string).collect()),
            ..ValidationConfig::default()
        }
    }

    #[test]
    fn test_allowed_types_validation() {
        let config = ValidationConfig {
            allowed_types: Some(vec!["feat".to_string(), "fix".to_string()]),
            ..ValidationConfig::default()
        };

        // Should pass: 'feat' is in the allowed list
        let res = validate_commit_with_config("feat: add feature", &config);
        assert!(res.is_ok());

        // Should fail: 'chore' is not in the allowed list
        let res = validate_commit_with_config("chore: update deps", &config);
        assert!(res.is_err());
        if let Err(CommitValidationError::TypeNotAllowed(type_name, list)) = res {
            assert_eq!(type_name, "chore");
            assert!(list.contains("feat, fix"));
        } else {
            panic!("Expected TypeNotAllowed error");
        }
    }

    #[test]
    fn test_allowed_types_with_custom_type() {
        let config = ValidationConfig {
            allowed_types: Some(vec!["api".to_string()]),
            ..ValidationConfig::default()
        };

        // Should pass: 'api' is explicitly allowed
        let res = validate_commit_with_config("api: new endpoint", &config);
        assert!(res.is_ok());

        // Should fail: standard 'feat' is not in this specific project's list
        let res = validate_commit_with_config("feat: add feature", &config);
        assert!(res.is_err());
    }

    #[test]
    fn test_scope_allowed() {
        let cfg = config_with_scopes(&["api", "client"]);
        assert!(validate_commit_with_config("feat(api): add endpoint", &cfg).is_ok());
        assert!(validate_commit_with_config("fix(client): handle timeout", &cfg).is_ok());
    }

    #[test]
    fn test_scope_not_allowed() {
        let cfg = config_with_scopes(&["api", "client"]);
        let err = validate_commit_with_config("feat(ui): add button", &cfg).unwrap_err();
        assert!(
            matches!(err, CommitValidationError::ScopeNotAllowed(ref scope) if scope == "ui"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_no_scope_passes_when_scopes_restricted() {
        // Commits without any scope are always accepted even when the list is set
        let cfg = config_with_scopes(&["api"]);
        assert!(validate_commit_with_config("feat: add thing", &cfg).is_ok());
    }

    #[test]
    fn test_no_scope_restriction_accepts_anything() {
        // Default config (allowed_scopes = None) accepts any scope
        assert!(validate_commit("feat(whatever): add thing").is_ok());
    }

    // ---- issue-pattern tests ----

    fn config_with_issue_pattern(pattern: &str) -> ValidationConfig {
        ValidationConfig {
            issue_pattern: Some(pattern.to_string()),
            ..ValidationConfig::default()
        }
    }

    #[test]
    fn test_issue_pattern_matched_by_footer() {
        let cfg = config_with_issue_pattern(r"^Refs: #\d+");
        let msg = "feat: add feature\n\nRefs: #123";
        assert!(validate_commit_with_config(msg, &cfg).is_ok());
    }

    #[test]
    fn test_issue_pattern_matched_jira_style() {
        let cfg = config_with_issue_pattern(r"^Fixes: [A-Z]+-\d+");
        let msg = "fix: resolve crash\n\nFixes: PROJ-456";
        assert!(validate_commit_with_config(msg, &cfg).is_ok());
    }

    #[test]
    fn test_issue_pattern_no_matching_footer_fails() {
        let cfg = config_with_issue_pattern(r"^Refs: #\d+");
        let msg = "feat: add feature\n\nReviewed-by: Alice";
        let err = validate_commit_with_config(msg, &cfg).unwrap_err();
        assert!(
            matches!(err, CommitValidationError::MissingIssueRef(_)),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_issue_pattern_no_footers_at_all_fails() {
        let cfg = config_with_issue_pattern(r"^Refs: #\d+");
        let msg = "feat: add feature";
        let err = validate_commit_with_config(msg, &cfg).unwrap_err();
        assert!(
            matches!(err, CommitValidationError::MissingIssueRef(_)),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_issue_pattern_none_always_passes() {
        // Default config has no issue_pattern — any valid commit passes
        assert!(validate_commit("feat: add feature").is_ok());
        assert!(validate_commit("fix: resolve crash\n\nReviewed-by: Alice").is_ok());
    }

    #[test]
    fn test_issue_pattern_invalid_regex_returns_error() {
        let cfg = config_with_issue_pattern(r"[invalid(");
        let msg = "feat: add feature\n\nRefs: #123";
        let err = validate_commit_with_config(msg, &cfg).unwrap_err();
        assert!(
            matches!(err, CommitValidationError::InvalidIssuePattern(_, _)),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_issue_pattern_one_of_multiple_footers_matches() {
        let cfg = config_with_issue_pattern(r"^Closes: #\d+");
        let msg = "feat: add feature\n\nReviewed-by: Bob\nCloses: #42";
        assert!(validate_commit_with_config(msg, &cfg).is_ok());
    }

    // ---- require-scope tests ----

    #[test]
    fn test_require_scope_missing() {
        let cfg = ValidationConfig {
            require_scope: true,
            ..ValidationConfig::default()
        };
        let err = validate_commit_with_config("feat: add thing", &cfg).unwrap_err();
        assert!(
            matches!(err, CommitValidationError::ScopeRequired),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_require_scope_present_passes() {
        let cfg = ValidationConfig {
            require_scope: true,
            ..ValidationConfig::default()
        };
        assert!(validate_commit_with_config("feat(api): add thing", &cfg).is_ok());
    }

    #[test]
    fn test_require_scope_not_enforced_by_default() {
        assert!(validate_commit("feat: add thing").is_ok());
    }

    // ---- scope-lowercase tests ----

    #[test]
    fn test_scope_lowercase_enforced() {
        let cfg = ValidationConfig {
            enforce_lowercase_scope: true,
            ..ValidationConfig::default()
        };
        let err = validate_commit_with_config("feat(API): add endpoint", &cfg).unwrap_err();
        assert!(
            matches!(err, CommitValidationError::ScopeNotLowercase(ref scope) if scope == "API"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_scope_lowercase_not_enforced_by_default() {
        assert!(validate_commit("feat(API): add endpoint").is_ok());
    }

    #[test]
    fn test_scope_already_lowercase_passes() {
        let cfg = ValidationConfig {
            enforce_lowercase_scope: true,
            ..ValidationConfig::default()
        };
        assert!(validate_commit_with_config("feat(api): add endpoint", &cfg).is_ok());
    }

    // ---- multiple-errors tests ----

    #[test]
    fn test_multiple_errors_collected() {
        // Uppercase first letter + trailing period → two description errors reported at once
        let err = validate_commit("feat: Add thing.").unwrap_err();
        assert!(
            matches!(err, CommitValidationError::Multiple(_)),
            "expected Multiple, got: {err}"
        );
    }

    #[test]
    fn test_single_error_not_wrapped_in_multiple() {
        // Only one violation should not be wrapped in Multiple
        let err = validate_commit("feat: Add thing").unwrap_err();
        assert!(
            matches!(err, CommitValidationError::DescriptionNotLowercase),
            "expected DescriptionNotLowercase, got: {err}"
        );
    }

    // ---- min-description-length tests ----

    #[test]
    fn test_min_description_length_too_short() {
        let cfg = ValidationConfig {
            min_description_length: 20,
            ..ValidationConfig::default()
        };
        let err = validate_commit_with_config("feat: add thing", &cfg).unwrap_err();
        assert!(
            matches!(err, CommitValidationError::DescriptionTooShort(20)),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_min_description_length_ok() {
        let cfg = ValidationConfig {
            min_description_length: 5,
            ..ValidationConfig::default()
        };
        assert!(validate_commit_with_config("feat: add thing", &cfg).is_ok());
    }

    #[test]
    fn test_min_description_length_zero_default() {
        // Default config has no minimum — single-char description passes
        assert!(validate_commit("fix: x").is_ok());
    }
}
