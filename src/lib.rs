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
            || self.footers.iter().any(|f| {
                f.starts_with("BREAKING CHANGE:") || f.starts_with("BREAKING-CHANGE:")
            })
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
    let re = Regex::new(r"^(?P<type>\w+)(?:\((?P<scope>[^)]+)\))?(?P<breaking>!)?:\s*(?P<description>.+)$")
        .unwrap();

    let captures = re
        .captures(header)
        .ok_or(CommitValidationError::InvalidFormat)?;

    let type_str = captures.name("type").unwrap().as_str();
    let scope = captures.name("scope").map(|m| m.as_str().to_string());
    let breaking_change = captures.name("breaking").is_some();
    let description = captures.name("description").unwrap().as_str();

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
    let body = if lines.len() > 1 && !lines[1].trim().is_empty() {
        None // First non-empty line after header is start of body
    } else if lines.len() > 2 {
        let body_lines: Vec<&str> = lines[2..]
            .iter()
            .take_while(|&&line| !line.starts_with("BREAKING CHANGE:") && !line.starts_with("BREAKING-CHANGE:"))
            .cloned()
            .collect();

        if body_lines.is_empty() {
            None
        } else {
            Some(body_lines.join("\n"))
        }
    } else {
        None
    };

    // Parse footers
    let footers: Vec<String> = lines
        .iter()
        .filter(|line| {
            line.starts_with("BREAKING CHANGE:")
                || line.starts_with("BREAKING-CHANGE:")
                || line.contains(": ")
        })
        .map(|s| s.to_string())
        .collect();

    Ok(ConventionalCommit {
        commit_type,
        scope,
        breaking_change,
        description: description.to_string(),
        body,
        footers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_commits() {
        let valid_commits = vec![
            "feat: add user authentication",
            "fix: resolve login issue",
            "feat(auth): add OAuth support",
            "feat!: breaking change to API",
            "chore: update dependencies",
            "docs: update README",
        ];

        for commit in valid_commits {
            assert!(validate_commit(commit).is_ok(), "Failed to validate: {}", commit);
        }
    }

    #[test]
    fn test_invalid_commits() {
        let invalid_commits = vec![
            "",
            "add user authentication",
            "feat:",
            "feat: Add user authentication", // Uppercase
            "feat: add user authentication.", // Period
        ];

        for commit in invalid_commits {
            assert!(validate_commit(commit).is_err(), "Should have failed: {}", commit);
        }
    }

    #[test]
    fn test_breaking_change_detection() {
        let commit = validate_commit("feat!: breaking change").unwrap();
        assert!(commit.breaking_change);

        let commit_with_footer = validate_commit("feat: new feature\n\nBREAKING CHANGE: this breaks something").unwrap();
        assert!(commit_with_footer.is_breaking_change());
    }

    #[test]
    fn test_commit_types() {
        assert_eq!(CommitType::from("feat"), CommitType::Feat);
        assert_eq!(CommitType::from("fix"), CommitType::Fix);
        assert_eq!(CommitType::from("custom"), CommitType::Custom("custom".to_string()));
    }
}
