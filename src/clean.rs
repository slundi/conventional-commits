//! Removal of unwanted lines from a commit message body.

use regex::Regex;

/// Result of applying clean rules to a commit message body.
#[derive(Debug, Clone)]
pub struct CleanResult {
    /// The message after removing matched lines, collapsing excess blank lines,
    /// and trimming trailing whitespace.
    pub cleaned_message: String,
    /// Every line that was removed (in the order they appeared in the original message).
    pub removed_lines: Vec<String>,
}

/// Cleans a commit message body by removing lines that match any of the provided rules.
///
/// Rules are applied only to lines after the first (header). Cleaning steps:
/// 1. Remove lines where `line.trim_end()` starts with any entry in `starts_with_rules`.
/// 2. Remove lines where `line.trim_end()` matches any compiled regex in `regex_rules`.
/// 3. Trim trailing whitespace from every surviving line.
/// 4. Collapse consecutive blank lines down to at most one.
/// 5. Strip trailing blank lines.
///
/// Returns `Err` if any regex pattern fails to compile.
pub fn clean_commit_body(
    message: &str,
    starts_with_rules: &[&str],
    regex_rules: &[&str],
) -> Result<CleanResult, regex::Error> {
    let compiled: Vec<Regex> = regex_rules
        .iter()
        .map(|pattern| Regex::new(pattern))
        .collect::<Result<Vec<_>, _>>()?;

    let lines: Vec<&str> = message.lines().collect();
    if lines.len() <= 1 || (starts_with_rules.is_empty() && compiled.is_empty()) {
        return Ok(CleanResult {
            cleaned_message: message.to_string(),
            removed_lines: vec![],
        });
    }

    let header = lines[0].to_string();
    let mut removed_lines: Vec<String> = Vec::new();
    let mut kept: Vec<String> = Vec::new();

    for line in &lines[1..] {
        let trimmed = line.trim_end();
        let matches = starts_with_rules
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
            || compiled.iter().any(|regex| regex.is_match(trimmed));
        if matches {
            removed_lines.push(line.to_string());
        } else {
            kept.push(trimmed.to_string());
        }
    }

    // Collapse consecutive blank lines to at most one
    let mut collapsed: Vec<String> = Vec::new();
    let mut last_blank = false;
    for line in kept {
        let blank = line.is_empty();
        if blank && last_blank {
            continue;
        }
        last_blank = blank;
        collapsed.push(line);
    }

    // Strip trailing blank lines
    while collapsed.last().is_some_and(String::is_empty) {
        collapsed.pop();
    }

    let mut result_lines = vec![header];
    result_lines.extend(collapsed);

    Ok(CleanResult {
        cleaned_message: result_lines.join("\n"),
        removed_lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_no_rules_returns_message_unchanged() {
        let msg = "feat: add thing\n\nBody line.\nCo-authored-by: Alice";
        let result = clean_commit_body(msg, &[], &[]).unwrap();
        assert_eq!(result.cleaned_message, msg);
        assert!(result.removed_lines.is_empty());
    }

    #[test]
    fn clean_starts_with_removes_matching_lines() {
        let msg =
            "feat: add thing\n\nBody text.\nCo-authored-by: Alice <a@x.com>\nSigned-off-by: Bob";
        let result = clean_commit_body(msg, &["Co-authored-by", "Signed-off-by"], &[]).unwrap();
        assert_eq!(result.cleaned_message, "feat: add thing\n\nBody text.");
        assert_eq!(result.removed_lines.len(), 2);
        assert!(result.removed_lines[0].contains("Co-authored-by"));
        assert!(result.removed_lines[1].contains("Signed-off-by"));
    }

    #[test]
    fn clean_regex_removes_matching_lines() {
        let msg = "fix: crash\n\nFixed the thing.\nCo-authored-by: Alice <a@x.com>";
        let result = clean_commit_body(msg, &[], &[r"^Co-authored-by:"]).unwrap();
        assert_eq!(result.cleaned_message, "fix: crash\n\nFixed the thing.");
        assert_eq!(result.removed_lines.len(), 1);
    }

    #[test]
    fn clean_both_rule_types_combined() {
        let msg = "feat: thing\n\nBody.\nCo-authored-by: Alice\nSigned-off-by: Bob\n🤖 generated";
        let result =
            clean_commit_body(msg, &["Co-authored-by", "Signed-off-by"], &[r"^🤖"]).unwrap();
        assert_eq!(result.cleaned_message, "feat: thing\n\nBody.");
        assert_eq!(result.removed_lines.len(), 3);
    }

    #[test]
    fn clean_collapses_consecutive_blank_lines() {
        let msg = "feat: thing\n\nParagraph one.\n\n\nParagraph two.";
        let result = clean_commit_body(msg, &["__no_match__"], &[]).unwrap();
        assert_eq!(
            result.cleaned_message,
            "feat: thing\n\nParagraph one.\n\nParagraph two."
        );
    }

    #[test]
    fn clean_strips_trailing_blank_lines() {
        let msg = "feat: thing\n\nBody.\n\n";
        let result = clean_commit_body(msg, &["__no_match__"], &[]).unwrap();
        assert_eq!(result.cleaned_message, "feat: thing\n\nBody.");
    }

    #[test]
    fn clean_trims_trailing_whitespace_per_line() {
        let msg = "feat: thing\n\nLine with spaces.   \nAnother line.  ";
        let result = clean_commit_body(msg, &["__no_match__"], &[]).unwrap();
        assert_eq!(
            result.cleaned_message,
            "feat: thing\n\nLine with spaces.\nAnother line."
        );
    }

    #[test]
    fn clean_header_only_message_is_unchanged() {
        let msg = "feat: add thing";
        let result = clean_commit_body(msg, &["Co-authored-by"], &[]).unwrap();
        assert_eq!(result.cleaned_message, msg);
        assert!(result.removed_lines.is_empty());
    }

    #[test]
    fn clean_invalid_regex_returns_error() {
        let msg = "feat: thing\n\nBody.";
        let err = clean_commit_body(msg, &[], &[r"[invalid("]);
        assert!(err.is_err());
    }

    #[test]
    fn clean_does_not_touch_header() {
        // Even if the header would match a rule, it must not be removed.
        let msg = "feat: add thing\n\nCo-authored-by: Alice";
        let result = clean_commit_body(msg, &["feat"], &[]).unwrap();
        // header line starts with "feat" but must be kept
        assert!(result.cleaned_message.starts_with("feat: add thing"));
    }

    #[test]
    fn clean_preserves_message_when_no_lines_removed() {
        let msg = "fix: resolve crash\n\nFixed the null pointer.\n\nRefs: #42";
        let result = clean_commit_body(msg, &["Co-authored-by"], &[]).unwrap();
        assert_eq!(result.cleaned_message, msg);
        assert!(result.removed_lines.is_empty());
    }
}
