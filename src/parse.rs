//! Low-level parsing of commit headers and footers.

use crate::error::CommitValidationError;

/// Parsed header parts: `(type, scope, breaking, description)`.
pub(crate) type HeaderParts = (String, Option<String>, bool, String);

/// Parses a commit header strictly: requires exactly one space after `:`.
pub(crate) fn parse_header(header: &str) -> Result<HeaderParts, CommitValidationError> {
    let bytes = header.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
        pos += 1;
    }
    if pos == 0 {
        return Err(CommitValidationError::MissingType);
    }
    let type_str = header[..pos].to_string();

    let scope = if bytes.get(pos) == Some(&b'(') {
        pos += 1;
        let start = pos;
        while pos < bytes.len() && bytes[pos] != b')' {
            pos += 1;
        }
        if pos >= bytes.len() {
            return Err(CommitValidationError::InvalidFormat);
        }
        let scope = header[start..pos].to_string();
        pos += 1;
        Some(scope)
    } else {
        None
    };

    let breaking = if bytes.get(pos) == Some(&b'!') {
        pos += 1;
        true
    } else {
        false
    };

    if bytes.get(pos) != Some(&b':') || bytes.get(pos + 1) != Some(&b' ') {
        return Err(CommitValidationError::InvalidFormat);
    }
    pos += 2;

    let description = header[pos..].to_string();
    if description.is_empty() {
        return Err(CommitValidationError::MissingDescription);
    }

    Ok((type_str, scope, breaking, description))
}

/// Parses a commit header loosely: allows zero or more spaces after `:` (for auto-fix).
pub(crate) fn parse_header_loose(header: &str) -> Option<HeaderParts> {
    let bytes = header.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
        pos += 1;
    }
    if pos == 0 {
        return None;
    }
    let type_str = header[..pos].to_string();

    let scope = if bytes.get(pos) == Some(&b'(') {
        pos += 1;
        let start = pos;
        while pos < bytes.len() && bytes[pos] != b')' {
            pos += 1;
        }
        if pos >= bytes.len() {
            return None;
        }
        let scope = header[start..pos].to_string();
        pos += 1;
        Some(scope)
    } else {
        None
    };

    let breaking = if bytes.get(pos) == Some(&b'!') {
        pos += 1;
        true
    } else {
        false
    };

    if bytes.get(pos) != Some(&b':') {
        return None;
    }
    pos += 1;

    while pos < bytes.len() && bytes[pos] == b' ' {
        pos += 1;
    }

    let description = header[pos..].to_string();
    if description.is_empty() {
        return None;
    }

    Some((type_str, scope, breaking, description))
}

/// Helper to identify Git trailers (footers): "Token: Value" or "Token #Value"
pub(crate) fn is_footer(line: &str) -> bool {
    let token_end = line
        .find(|character: char| !character.is_ascii_alphabetic() && character != '-')
        .unwrap_or(0);
    if token_end == 0 {
        return false;
    }
    let rest = &line[token_end..];
    (rest.starts_with(": ") && rest.len() > 2) || rest.starts_with(" #")
}

pub(crate) fn is_breaking_footer(line: &str) -> bool {
    line.starts_with("BREAKING CHANGE: ") || line.starts_with("BREAKING-CHANGE: ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(
        type_str: &str,
        scope: Option<&str>,
        breaking: bool,
        description: &str,
    ) -> HeaderParts {
        (
            type_str.to_string(),
            scope.map(ToString::to_string),
            breaking,
            description.to_string(),
        )
    }

    #[test]
    fn test_parse_header_full() {
        assert_eq!(
            parse_header("feat(api)!: add endpoint").unwrap(),
            parts("feat", Some("api"), true, "add endpoint")
        );
    }

    #[test]
    fn test_parse_header_minimal() {
        assert_eq!(
            parse_header("fix: bug").unwrap(),
            parts("fix", None, false, "bug")
        );
    }

    #[test]
    fn test_parse_header_keeps_extra_spaces_in_description() {
        assert_eq!(parse_header("fix:  bug").unwrap().3, " bug");
    }

    #[test]
    fn test_parse_header_errors() {
        assert!(matches!(
            parse_header(": no type"),
            Err(CommitValidationError::MissingType)
        ));
        assert!(matches!(
            parse_header("feat(api: unclosed"),
            Err(CommitValidationError::InvalidFormat)
        ));
        assert!(matches!(
            parse_header("feat:no-space"),
            Err(CommitValidationError::InvalidFormat)
        ));
        assert!(matches!(
            parse_header("feat add"),
            Err(CommitValidationError::InvalidFormat)
        ));
        assert!(matches!(
            parse_header("feat: "),
            Err(CommitValidationError::MissingDescription)
        ));
    }

    #[test]
    fn test_parse_header_loose_accepts_any_spacing() {
        assert_eq!(
            parse_header_loose("feat:add").unwrap(),
            parts("feat", None, false, "add")
        );
        assert_eq!(
            parse_header_loose("feat(ui)!:   add").unwrap(),
            parts("feat", Some("ui"), true, "add")
        );
    }

    #[test]
    fn test_parse_header_loose_rejects_invalid() {
        assert!(parse_header_loose(": no type").is_none());
        assert!(parse_header_loose("feat(api: unclosed").is_none());
        assert!(parse_header_loose("feat add").is_none());
        assert!(parse_header_loose("feat:   ").is_none());
    }

    #[test]
    fn test_is_footer() {
        assert!(is_footer("Refs: #123"));
        assert!(is_footer("Reviewed-by: Alice"));
        assert!(is_footer("Closes #42"));
        assert!(!is_footer("Refs: "));
        assert!(!is_footer("plain body line"));
        assert!(!is_footer(": value"));
        assert!(!is_footer(""));
    }

    #[test]
    fn test_is_breaking_footer() {
        assert!(is_breaking_footer("BREAKING CHANGE: api changed"));
        assert!(is_breaking_footer("BREAKING-CHANGE: api changed"));
        assert!(!is_breaking_footer("BREAKING CHANGE:no-space"));
        assert!(!is_breaking_footer("breaking change: lower"));
    }
}
