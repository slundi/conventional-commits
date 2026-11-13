//! Validation error types.

use std::fmt;
use thiserror::Error;

/// A list of commit validation errors, used when multiple issues are detected simultaneously.
#[derive(Debug)]
pub struct ValidationErrors(pub Vec<CommitValidationError>);

impl fmt::Display for ValidationErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, error) in self.0.iter().enumerate() {
            if index > 0 {
                writeln!(formatter)?;
            }
            write!(formatter, "  - {}", error)?;
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

    #[error("Description is too short (min {0} characters)")]
    DescriptionTooShort(usize),

    #[error("Commit type '{0}' is not recognized")]
    UnknownType(String),

    #[error("Type '{0}' is not allowed. Allowed types are: {1}")]
    TypeNotAllowed(String, String),

    #[error("A blank line is required between the header and the body")]
    MissingBlankLine,

    #[error("Scope '{0}' is not in the allowed scopes list")]
    ScopeNotAllowed(String),

    #[error("Scope '{0}' must be lowercase")]
    ScopeNotLowercase(String),

    #[error("A scope is required")]
    ScopeRequired,

    #[error("No footer matches the required issue pattern '{0}'")]
    MissingIssueRef(String),

    #[error("Invalid issue pattern regex '{0}': {1}")]
    InvalidIssuePattern(String, String),

    #[error("Multiple validation errors:\n{0}")]
    Multiple(ValidationErrors),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_errors_display_one_per_line() {
        let errors = ValidationErrors(vec![
            CommitValidationError::ScopeRequired,
            CommitValidationError::DescriptionEndsWithPeriod,
        ]);
        assert_eq!(
            errors.to_string(),
            "  - A scope is required\n  - Description must not end with a period"
        );
    }

    #[test]
    fn test_validation_errors_display_empty() {
        assert_eq!(ValidationErrors(vec![]).to_string(), "");
    }

    #[test]
    fn test_multiple_error_display_includes_header() {
        let error = CommitValidationError::Multiple(ValidationErrors(vec![
            CommitValidationError::MissingType,
        ]));
        assert_eq!(
            error.to_string(),
            "Multiple validation errors:\n  - Missing commit type"
        );
    }
}
