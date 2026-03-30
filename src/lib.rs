//! # Conventional Commits Validator
//!
//! A lightweight library for validating Git commit messages according to the
//! [Conventional Commits](https://conventionalcommits.org/) specification.
//!
//! ## Usage
//!
//! ```rust
//! use conventional_commits::{validate_commit, CommitType};
//!
//! let message = "feat: add new user authentication";
//! match validate_commit(message) {
//!     Ok(commit) => {
//!         println!("Valid commit: {:?}", commit.commit_type);
//!     }
//!     Err(e) => {
//!         eprintln!("Invalid commit: {}", e);
//!     }
//! }
//! ```

use regex::Regex;
use std::fmt;
use thiserror::Error;

/// Represents the type of a conventional commit
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitType {
    /// A new feature
    Feat,
    /// A bug fix
    Fix,
    /// Documentation only changes
    Docs,
    /// Changes that do not affect the meaning of the code
    Style,
    /// A code change that neither fixes a bug nor adds a feature
    Refactor,
    /// A code change that improves performance
    Perf,
    /// Adding missing tests or correcting existing tests
    Test,
    /// Changes to the build process or auxiliary tools and libraries
    Build,
    /// Changes to CI configuration files and scripts
    Ci,
    /// Other changes that don't modify src or test files
    Chore,
    /// Reverts a previous commit
    Revert,
    /// Custom type not in the standard list
    Custom(String),
}

impl fmt::Display for CommitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitType::Feat => write!(f, "feat"),
            CommitType::Fix => write!(f, "fix"),
            CommitType::Docs => write!(f, "docs"),
            CommitType::Style => write!(f, "style"),
            CommitType::Refactor => write!(f, "refactor"),
            CommitType::Perf => write!(f, "perf"),
            CommitType::Test => write!(f, "test"),
            CommitType::Build => write!(f, "build"),
            CommitType::Ci => write!(f, "ci"),
            CommitType::Chore => write!(f, "chore"),
            CommitType::Revert => write!(f, "revert"),
            CommitType::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl From<&str> for CommitType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "feat" => CommitType::Feat,
            "fix" => CommitType::Fix,
            "docs" => CommitType::Docs,
            "style" => CommitType::Style,
            "refactor" => CommitType::Refactor,
            "perf" => CommitType::Perf,
            "test" => CommitType::Test,
            "build" => CommitType::Build,
            "ci" => CommitType::Ci,
            "chore" => CommitType::Chore,
            "revert" => CommitType::Revert,
            _ => CommitType::Custom(s.to_string()),
        }
    }
}

/// Represents a parsed conventional commit
#[derive(Debug, Clone)]
pub struct ConventionalCommit {
    /// The type of the commit
    pub commit_type: CommitType,
    /// Optional scope of the commit
    pub scope: Option<String>,
    /// Whether this commit contains breaking changes
    pub breaking_change: bool,
    /// The description of the commit
    pub description: String,
    /// Optional body of the commit message
    pub body: Option<String>,
    /// Optional footers of the commit message
    pub footers: Vec<String>,
}

impl ConventionalCommit {
    /// Returns true if this commit represents a breaking change
    pub fn is_breaking_change(&self) -> bool {
        self.breaking_change
            || self
                .footers
                .iter()
                .any(|f| f.starts_with("BREAKING CHANGE:") || f.starts_with("BREAKING-CHANGE:"))
    }
}

/// A list of commit validation errors, used when multiple issues are detected simultaneously.
#[derive(Debug)]
pub struct ValidationErrors(pub Vec<CommitValidationError>);

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, e) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "  - {}", e)?;
        }
        Ok(())
    }
}

/// Errors that can occur when validating commit messages
#[derive(Error, Debug)]
pub enum CommitValidationError {
    #[error("Commit message is empty")]
    EmptyMessage,

    #[error("Invalid format: expected 'type(scope): description'")]
    InvalidFormat,

    #[error("Missing commit type")]
    MissingType,

    #[error("Missing description")]
    MissingDescription,

    #[error("Description must start with lowercase letter")]
    DescriptionNotLowercase,

    #[error("Description must not end with a period")]
    DescriptionEndsWithPeriod,

    #[error("Description is too long (max 72 characters)")]
    DescriptionTooLong,

    #[error("Commit type '{0}' is not recognized")]
    UnknownType(String),

    #[error("A blank line is required between the header and the body")]
    MissingBlankLine,

    #[error("Scope '{0}' is not in the allowed scopes list")]
    ScopeNotAllowed(String),

    #[error("No footer matches the required issue pattern '{0}'")]
    MissingIssueRef(String),

    #[error("Invalid issue pattern regex '{0}': {1}")]
    InvalidIssuePattern(String, String),

    #[error("Multiple validation errors:\n{0}")]
    Multiple(ValidationErrors),
}

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
    /// Optional regex pattern that at least one footer must match (e.g. `r"^Refs: #\d+"`)
    pub issue_pattern: Option<String>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_description_length: 72,
            allow_custom_types: true,
            enforce_lowercase_description: true,
            disallow_description_period: true,
            allowed_scopes: None,
            issue_pattern: None,
        }
    }
}

/// Parses a commit header strictly: requires exactly one space after `:`.
fn parse_header(
    header: &str,
) -> Result<(String, Option<String>, bool, String), CommitValidationError> {
    let bytes = header.as_bytes();
    let mut i = 0;

    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    if i == 0 {
        return Err(CommitValidationError::MissingType);
    }
    let type_str = header[..i].to_string();

    let scope = if bytes.get(i) == Some(&b'(') {
        i += 1;
        let start = i;
        while i < bytes.len() && bytes[i] != b')' {
            i += 1;
        }
        if i >= bytes.len() {
            return Err(CommitValidationError::InvalidFormat);
        }
        let sc = header[start..i].to_string();
        i += 1;
        Some(sc)
    } else {
        None
    };

    let breaking = if bytes.get(i) == Some(&b'!') {
        i += 1;
        true
    } else {
        false
    };

    if bytes.get(i) != Some(&b':') || bytes.get(i + 1) != Some(&b' ') {
        return Err(CommitValidationError::InvalidFormat);
    }
    i += 2;

    let description = header[i..].to_string();
    if description.is_empty() {
        return Err(CommitValidationError::MissingDescription);
    }

    Ok((type_str, scope, breaking, description))
}

/// Parses a commit header loosely: allows zero or more spaces after `:` (for auto-fix).
fn parse_header_loose(header: &str) -> Option<(String, Option<String>, bool, String)> {
    let bytes = header.as_bytes();
    let mut i = 0;

    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    let type_str = header[..i].to_string();

    let scope = if bytes.get(i) == Some(&b'(') {
        i += 1;
        let start = i;
        while i < bytes.len() && bytes[i] != b')' {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        let sc = header[start..i].to_string();
        i += 1;
        Some(sc)
    } else {
        None
    };

    let breaking = if bytes.get(i) == Some(&b'!') {
        i += 1;
        true
    } else {
        false
    };

    if bytes.get(i) != Some(&b':') {
        return None;
    }
    i += 1;

    while i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }

    let description = header[i..].to_string();
    if description.is_empty() {
        return None;
    }

    Some((type_str, scope, breaking, description))
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
    if !config.allow_custom_types {
        if let CommitType::Custom(_) = commit_type {
            errors.push(CommitValidationError::UnknownType(type_str.clone()));
        }
    }

    // Validate scope against allowed list (only when a scope is present)
    if let (Some(ref s), Some(ref allowed)) = (&scope, &config.allowed_scopes) {
        if !allowed.iter().any(|a| a == s) {
            errors.push(CommitValidationError::ScopeNotAllowed(s.clone()));
        }
    }

    // Validate description
    if config.enforce_lowercase_description {
        if let Some(first) = description.chars().next() {
            if !first.is_lowercase() {
                errors.push(CommitValidationError::DescriptionNotLowercase);
            }
        }
    }

    if config.disallow_description_period && description.ends_with('.') {
        errors.push(CommitValidationError::DescriptionEndsWithPeriod);
    }

    if description.len() > config.max_description_length {
        errors.push(CommitValidationError::DescriptionTooLong);
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
        let re = Regex::new(pattern).map_err(|e| {
            CommitValidationError::InvalidIssuePattern(pattern.clone(), e.to_string())
        })?;
        if !footer_lines.iter().any(|f| re.is_match(f)) {
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

/// Attempts to fix common mistakes in a commit message header.
///
/// Fixes applied (only to the first line):
/// - Adds missing space after colon (`feat:add thing` → `feat: add thing`)
/// - Lowercases first letter of description (`feat: Add thing` → `feat: add thing`)
/// - Removes trailing period from description (`feat: add thing.` → `feat: add thing`)
///
/// Returns the (potentially modified) message. If the header cannot be parsed
/// even loosely, the original message is returned unchanged.
pub fn fix_commit_message(message: &str) -> String {
    let lines: Vec<&str> = message.lines().collect();
    if lines.is_empty() {
        return message.to_string();
    }

    let header = lines[0];

    // Loose regex: allow zero or more spaces after colon so we can fix missing space
    let fixed_header = match parse_header_loose(header) {
        None => return message.to_string(),
        Some((type_str, scope, breaking, description)) => {
            let breaking_str = if breaking { "!" } else { "" };
            let fixed_desc = fix_description(&description);
            match scope {
                Some(s) => format!("{}({}){}: {}", type_str, s, breaking_str, fixed_desc),
                None => format!("{}{}: {}", type_str, breaking_str, fixed_desc),
            }
        }
    };

    if lines.len() == 1 {
        fixed_header
    } else {
        format!("{}\n{}", fixed_header, lines[1..].join("\n"))
    }
}

fn fix_description(desc: &str) -> String {
    // Strip trailing period
    let desc = desc.strip_suffix('.').unwrap_or(desc);

    // Lowercase first char
    let mut chars = desc.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let lower: String = first.to_lowercase().collect();
            lower + chars.as_str()
        }
    }
}

/// Helper to identify Git trailers (footers): "Token: Value" or "Token #Value"
fn is_footer(line: &str) -> bool {
    let token_end = line
        .find(|c: char| !c.is_ascii_alphabetic() && c != '-')
        .unwrap_or(0);
    if token_end == 0 {
        return false;
    }
    let rest = &line[token_end..];
    (rest.starts_with(": ") && rest.len() > 2) || rest.starts_with(" #")
}

fn is_breaking_footer(line: &str) -> bool {
    line.starts_with("BREAKING CHANGE: ") || line.starts_with("BREAKING-CHANGE: ")
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

    #[test]
    fn test_commit_types() {
        assert_eq!(CommitType::from("feat"), CommitType::Feat);
        assert_eq!(CommitType::from("fix"), CommitType::Fix);
        assert_eq!(
            CommitType::from("custom"),
            CommitType::Custom("custom".to_string())
        );
    }

    #[test]
    fn test_fix_missing_space() {
        assert_eq!(fix_commit_message("feat:add thing"), "feat: add thing");
        assert_eq!(fix_commit_message("fix:resolve bug"), "fix: resolve bug");
    }

    #[test]
    fn test_fix_uppercase_description() {
        assert_eq!(fix_commit_message("feat: Add thing"), "feat: add thing");
        assert_eq!(fix_commit_message("fix: Resolve bug"), "fix: resolve bug");
    }

    #[test]
    fn test_fix_trailing_period() {
        assert_eq!(fix_commit_message("feat: add thing."), "feat: add thing");
        assert_eq!(fix_commit_message("fix: resolve bug."), "fix: resolve bug");
    }

    #[test]
    fn test_fix_all_at_once() {
        assert_eq!(fix_commit_message("feat:Add thing."), "feat: add thing");
        assert_eq!(
            fix_commit_message("fix(parser):Resolve bug."),
            "fix(parser): resolve bug"
        );
    }

    #[test]
    fn test_fix_preserves_valid_message() {
        let msg = "feat: add thing";
        assert_eq!(fix_commit_message(msg), msg);
    }

    #[test]
    fn test_fix_with_scope_and_breaking() {
        assert_eq!(
            fix_commit_message("feat(auth)!:Add OAuth."),
            "feat(auth)!: add OAuth"
        );
    }

    #[test]
    fn test_fix_preserves_body() {
        let msg = "feat:Add thing.\n\nThis is the body.";
        let fixed = fix_commit_message(msg);
        assert_eq!(fixed, "feat: add thing\n\nThis is the body.");
    }

    #[test]
    fn test_fix_unparseable_returns_original() {
        let msg = "not a conventional commit at all";
        assert_eq!(fix_commit_message(msg), msg);
    }

    // ---- scope-enum tests ----

    fn config_with_scopes(scopes: &[&str]) -> ValidationConfig {
        ValidationConfig {
            allowed_scopes: Some(scopes.iter().map(|s| s.to_string()).collect()),
            ..ValidationConfig::default()
        }
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
            matches!(err, CommitValidationError::ScopeNotAllowed(ref s) if s == "ui"),
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
}
