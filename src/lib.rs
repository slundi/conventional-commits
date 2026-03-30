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
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_description_length: 72,
            allow_custom_types: true,
            enforce_lowercase_description: true,
            disallow_description_period: true,
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

    // Parse the header using regex
    let re = Regex::new(
        r"^(?P<type>\w+)(?:\((?P<scope>[^)]+)\))?(?P<breaking>!)?:\s+(?P<description>.+)$",
    )
    .unwrap();

    let captures = re
        .captures(header)
        .ok_or(CommitValidationError::InvalidFormat)?;

    let type_str = captures.name("type").unwrap().as_str();
    let scope = captures.name("scope").map(|m| m.as_str().to_string());
    let breaking_change = captures.name("breaking").is_some();
    let description = captures.name("description").unwrap().as_str();

    // Validate Blank Line Rule:
    // If there is more than one line, the second line MUST be empty.
    if lines.len() > 1 && !lines[1].trim().is_empty() {
        return Err(CommitValidationError::MissingBlankLine);
    }

    // Validate commit type
    let commit_type = CommitType::from(type_str);
    if !config.allow_custom_types {
        if let CommitType::Custom(_) = commit_type {
            return Err(CommitValidationError::UnknownType(type_str.to_string()));
        }
    }

    // Validate description
    if description.is_empty() {
        return Err(CommitValidationError::MissingDescription);
    }

    if config.enforce_lowercase_description && !description.chars().next().unwrap().is_lowercase() {
        return Err(CommitValidationError::DescriptionNotLowercase);
    }

    if config.disallow_description_period && description.ends_with('.') {
        return Err(CommitValidationError::DescriptionEndsWithPeriod);
    }

    if description.len() > config.max_description_length {
        return Err(CommitValidationError::DescriptionTooLong);
    }

    // Parse body and footers
    let body_parts: Vec<String> = Vec::new();
    let mut body_lines = Vec::new();
    let mut footer_lines = Vec::new();
    let mut is_breaking = breaking_change;

    if lines.len() > 2 {
        let iter = lines[2..].iter().peekable();

        // Separate Body from Footers
        for line in iter {
            // If the current line looks like a footer AND there are no non-footer lines after it
            if is_breaking_footer(line) || is_footer(line) {
                // We found a potential footer.
                // A stricter tool would ensure once footers start, body text cannot reappear.
                footer_lines.push(line.to_string());

                if is_breaking_footer(line) {
                    is_breaking = true;
                }
            } else {
                // If we already found "footers" but then find a normal line,
                // the previous "footers" were actually just part of the body.
                if !footer_lines.is_empty() {
                    body_lines.append(&mut footer_lines);
                }
                body_lines.push((*line).to_string());
            }
        }
    }

    let body = if body_parts.is_empty() {
        None
    } else {
        Some(body_lines.join("\n").trim().to_string())
    };

    Ok(ConventionalCommit {
        commit_type,
        scope,
        breaking_change: is_breaking,
        description: description.to_string(),
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
    let re = Regex::new(
        r"^(?P<type>\w+)(?:\((?P<scope>[^)]+)\))?(?P<breaking>!)?:\s*(?P<description>\S.*)$",
    )
    .unwrap();

    let fixed_header = match re.captures(header) {
        None => return message.to_string(),
        Some(caps) => {
            let type_str = caps.name("type").unwrap().as_str();
            let scope = caps.name("scope").map(|m| m.as_str());
            let breaking = if caps.name("breaking").is_some() {
                "!"
            } else {
                ""
            };
            let description = caps.name("description").unwrap().as_str();

            let fixed_desc = fix_description(description);

            match scope {
                Some(s) => format!("{}({}){}: {}", type_str, s, breaking, fixed_desc),
                None => format!("{}{}: {}", type_str, breaking, fixed_desc),
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

/// Helper to identify Git trailers (footers)
fn is_footer(line: &str) -> bool {
    // Basic check for "Token: Value" or "Token #Value"
    let re = Regex::new(r"^[a-zA-Z-]+: .|^[a-zA-Z-]+ #").unwrap();
    re.is_match(line)
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
}
