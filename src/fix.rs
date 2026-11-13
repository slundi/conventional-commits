//! Automatic fixing of common commit header mistakes.

use crate::parse::parse_header_loose;

/// Attempts to fix common mistakes in a commit message header.
///
/// Fixes applied (only to the first line):
/// - Lowercases the commit type (`Feat:` → `feat:`)
/// - Adds missing space after colon (`feat:add thing` → `feat: add thing`)
/// - Lowercases first letter of description (`feat: Add thing` → `feat: add thing`)
/// - Removes trailing period from description (`feat: add thing.` → `feat: add thing`)
/// - Strips leading/trailing whitespace from description (`feat:  add thing  ` → `feat: add thing`)
///
/// Returns the (potentially modified) message. If the header cannot be parsed
/// even loosely, the original message is returned unchanged.
pub fn fix_commit_message(message: &str) -> String {
    let lines: Vec<&str> = message.lines().collect();
    if lines.is_empty() {
        return message.to_string();
    }

    let header = lines[0];

    // Loose parse: allows zero or more spaces after colon so we can fix missing space
    let fixed_header = match parse_header_loose(header) {
        None => return message.to_string(),
        Some((type_str, scope, breaking, description)) => {
            let type_lower = type_str.to_lowercase();
            let breaking_str = if breaking { "!" } else { "" };
            let fixed_desc = fix_description(&description);
            match scope {
                Some(scope) => format!("{}({}){}: {}", type_lower, scope, breaking_str, fixed_desc),
                None => format!("{}{}: {}", type_lower, breaking_str, fixed_desc),
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
    // Strip leading/trailing whitespace
    let desc = desc.trim();
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_fix_type_casing() {
        assert_eq!(fix_commit_message("Feat: add thing"), "feat: add thing");
        assert_eq!(fix_commit_message("FIX: resolve bug"), "fix: resolve bug");
        assert_eq!(
            fix_commit_message("CHORE(deps): bump version"),
            "chore(deps): bump version"
        );
    }

    #[test]
    fn test_fix_description_whitespace() {
        assert_eq!(fix_commit_message("feat:   add thing  "), "feat: add thing");
        assert_eq!(
            fix_commit_message("fix:  resolve  bug  "),
            "fix: resolve  bug"
        );
    }

    #[test]
    fn test_fix_unparsable_returns_original() {
        let msg = "not a conventional commit at all";
        assert_eq!(fix_commit_message(msg), msg);
    }

    #[test]
    fn test_fix_empty_message_returns_original() {
        assert_eq!(fix_commit_message(""), "");
    }

    #[test]
    fn test_fix_description_only_period_becomes_empty() {
        assert_eq!(fix_description(" . "), "");
    }
}
