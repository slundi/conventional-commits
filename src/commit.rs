//! Data model of a parsed conventional commit.

use std::fmt;

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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitType::Feat => write!(formatter, "feat"),
            CommitType::Fix => write!(formatter, "fix"),
            CommitType::Docs => write!(formatter, "docs"),
            CommitType::Style => write!(formatter, "style"),
            CommitType::Refactor => write!(formatter, "refactor"),
            CommitType::Perf => write!(formatter, "perf"),
            CommitType::Test => write!(formatter, "test"),
            CommitType::Build => write!(formatter, "build"),
            CommitType::Ci => write!(formatter, "ci"),
            CommitType::Chore => write!(formatter, "chore"),
            CommitType::Revert => write!(formatter, "revert"),
            CommitType::Custom(name) => write!(formatter, "{}", name),
        }
    }
}

impl From<&str> for CommitType {
    fn from(name: &str) -> Self {
        match name.to_lowercase().as_str() {
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
            _ => CommitType::Custom(name.to_string()),
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
            || self.footers.iter().any(|footer| {
                footer.starts_with("BREAKING CHANGE:") || footer.starts_with("BREAKING-CHANGE:")
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STANDARD_TYPES: [(&str, CommitType); 11] = [
        ("feat", CommitType::Feat),
        ("fix", CommitType::Fix),
        ("docs", CommitType::Docs),
        ("style", CommitType::Style),
        ("refactor", CommitType::Refactor),
        ("perf", CommitType::Perf),
        ("test", CommitType::Test),
        ("build", CommitType::Build),
        ("ci", CommitType::Ci),
        ("chore", CommitType::Chore),
        ("revert", CommitType::Revert),
    ];

    fn commit_with(breaking_change: bool, footers: &[&str]) -> ConventionalCommit {
        ConventionalCommit {
            commit_type: CommitType::Feat,
            scope: None,
            breaking_change,
            description: "add thing".to_string(),
            body: None,
            footers: footers.iter().map(ToString::to_string).collect(),
        }
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
    fn test_commit_type_from_is_case_insensitive() {
        assert_eq!(CommitType::from("FEAT"), CommitType::Feat);
        assert_eq!(CommitType::from("Chore"), CommitType::Chore);
    }

    #[test]
    fn test_commit_type_display_roundtrip() {
        for (name, commit_type) in STANDARD_TYPES {
            assert_eq!(commit_type.to_string(), name);
            assert_eq!(CommitType::from(name), commit_type);
        }
        assert_eq!(CommitType::Custom("wip".to_string()).to_string(), "wip");
    }

    #[test]
    fn test_is_breaking_change_from_flag() {
        assert!(commit_with(true, &[]).is_breaking_change());
        assert!(!commit_with(false, &[]).is_breaking_change());
    }

    #[test]
    fn test_is_breaking_change_from_footer() {
        assert!(commit_with(false, &["BREAKING CHANGE: api"]).is_breaking_change());
        assert!(commit_with(false, &["BREAKING-CHANGE: api"]).is_breaking_change());
        assert!(!commit_with(false, &["Refs: #1"]).is_breaking_change());
    }
}
