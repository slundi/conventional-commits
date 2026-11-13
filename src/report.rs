//! Validation outcome and its rendering, kept separate from process I/O.
//!
//! [`evaluate`] decides the outcome, [`render_json`] / [`render_text`] turn it into output,
//! and the caller decides where to write it and which exit code to use.

use std::io;

use conventional_commits::{
    CommitValidationError, ConventionalCommit, ValidationConfig, validate_commit_with_config,
};
use termcolor::{Color, ColorSpec, WriteColor};

use crate::cli::HELP_HINT;

const MISSING_MESSAGE: &str = "No commit message provided";

/// Result of validating a commit message.
#[derive(Debug)]
pub(crate) enum Outcome {
    /// The commit message was empty.
    Missing,
    /// The commit message is a valid conventional commit.
    Valid(ConventionalCommit),
    /// The commit message failed validation.
    Invalid(CommitValidationError),
}

impl Outcome {
    /// Process exit code matching this outcome.
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            Outcome::Valid(_) => 0,
            Outcome::Missing | Outcome::Invalid(_) => 1,
        }
    }
}

/// Validates `message` against `config`.
pub(crate) fn evaluate(message: &str, config: &ValidationConfig) -> Outcome {
    if message.is_empty() {
        return Outcome::Missing;
    }
    match validate_commit_with_config(message, config) {
        Ok(commit) => Outcome::Valid(commit),
        Err(error) => Outcome::Invalid(error),
    }
}

/// Escapes a string for safe embedding in a JSON value.
fn json_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn json_string(input: &str) -> String {
    format!("\"{}\"", json_escape(input))
}

fn json_optional_string(input: Option<&str>) -> String {
    input.map_or_else(|| "null".to_string(), json_string)
}

fn json_string_array<S: AsRef<str>>(items: &[S]) -> String {
    let joined = items
        .iter()
        .map(|item| json_string(item.as_ref()))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

/// Flattens a `CommitValidationError` into individual error message strings.
/// `Multiple` is unpacked; all other variants become a single-element vec.
fn error_messages(error: &CommitValidationError) -> Vec<String> {
    match error {
        CommitValidationError::Multiple(errors) => {
            errors.0.iter().map(ToString::to_string).collect()
        }
        _ => vec![error.to_string()],
    }
}

/// Renders the outcome as a single-line JSON document (without trailing newline).
pub(crate) fn render_json(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Missing => format!(
            r#"{{"valid":false,"errors":{}}}"#,
            json_string_array(&[MISSING_MESSAGE])
        ),
        Outcome::Valid(commit) => format!(
            r#"{{"valid":true,"type":{},"scope":{},"description":{},"breaking_change":{},"body":{},"footers":{}}}"#,
            json_string(&commit.commit_type.to_string()),
            json_optional_string(commit.scope.as_deref()),
            json_string(&commit.description),
            commit.is_breaking_change(),
            json_optional_string(commit.body.as_deref()),
            json_string_array(&commit.footers),
        ),
        Outcome::Invalid(error) => format!(
            r#"{{"valid":false,"errors":{}}}"#,
            json_string_array(&error_messages(error))
        ),
    }
}

fn write_colored<W: WriteColor>(
    stream: &mut W,
    text: &str,
    color: Color,
    is_bold: bool,
) -> io::Result<()> {
    stream.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(is_bold))?;
    write!(stream, "{text}")?;
    stream.reset()
}

/// Renders the outcome as human-readable text.
/// Success details go to `out`; errors go to `err`.
pub(crate) fn render_text<O: WriteColor, E: WriteColor>(
    outcome: &Outcome,
    message: &str,
    verbose: bool,
    out: &mut O,
    err: &mut E,
) -> io::Result<()> {
    match outcome {
        Outcome::Missing => writeln!(err, "Error: {MISSING_MESSAGE}"),
        Outcome::Valid(commit) => {
            write_colored(out, "✅ Valid conventional commit!", Color::Green, true)?;
            writeln!(out)?;
            if !verbose {
                return writeln!(out, "✅ Valid conventional commit");
            }
            writeln!(out, "Type: {}", commit.commit_type)?;
            if let Some(scope) = &commit.scope {
                writeln!(out, "Scope: {scope}")?;
            }
            writeln!(out, "Description: {}", commit.description)?;
            if commit.is_breaking_change() {
                write_colored(out, "⚠️  BREAKING CHANGE detected!", Color::Yellow, true)?;
                writeln!(out)?;
            }
            if let Some(body) = &commit.body {
                writeln!(out, "Body: {body}")?;
            }
            if !commit.footers.is_empty() {
                writeln!(out, "Footers: {:?}", commit.footers)?;
            }
            Ok(())
        }
        Outcome::Invalid(error) => {
            write_colored(err, "❌ Invalid commit message: ", Color::Red, true)?;
            writeln!(err, "{error}")?;
            if verbose {
                writeln!(err, "Message was: '{message}'")?;
                write!(out, "{HELP_HINT}")?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termcolor::Buffer;

    fn evaluate_default(message: &str) -> Outcome {
        evaluate(message, &ValidationConfig::default())
    }

    /// Renders text output and returns `(stdout, stderr)` as strings.
    fn render_text_to_strings(outcome: &Outcome, message: &str, verbose: bool) -> (String, String) {
        let mut out = Buffer::no_color();
        let mut err = Buffer::no_color();
        render_text(outcome, message, verbose, &mut out, &mut err).expect("in-memory write");
        (
            String::from_utf8(out.into_inner()).expect("utf8 stdout"),
            String::from_utf8(err.into_inner()).expect("utf8 stderr"),
        )
    }

    #[test]
    fn test_evaluate_missing() {
        let outcome = evaluate_default("");
        assert!(matches!(outcome, Outcome::Missing));
        assert_eq!(outcome.exit_code(), 1);
    }

    #[test]
    fn test_evaluate_valid() {
        let outcome = evaluate_default("feat(api): add endpoint");
        assert!(matches!(outcome, Outcome::Valid(_)));
        assert_eq!(outcome.exit_code(), 0);
    }

    #[test]
    fn test_evaluate_invalid() {
        let outcome = evaluate_default("not a conventional commit");
        assert!(matches!(outcome, Outcome::Invalid(_)));
        assert_eq!(outcome.exit_code(), 1);
    }

    #[test]
    fn test_evaluate_respects_config() {
        let config = ValidationConfig {
            require_scope: true,
            ..ValidationConfig::default()
        };
        assert!(matches!(
            evaluate("feat: add endpoint", &config),
            Outcome::Invalid(_)
        ));
    }

    #[test]
    fn test_json_escape_special_chars() {
        assert_eq!(json_escape("a\"b\\c\nd\re\tf"), "a\\\"b\\\\c\\nd\\re\\tf");
    }

    #[test]
    fn test_render_json_missing() {
        assert_eq!(
            render_json(&Outcome::Missing),
            r#"{"valid":false,"errors":["No commit message provided"]}"#
        );
    }

    #[test]
    fn test_render_json_valid_minimal() {
        let outcome = evaluate_default("fix: resolve bug");
        assert_eq!(
            render_json(&outcome),
            r#"{"valid":true,"type":"fix","scope":null,"description":"resolve bug","breaking_change":false,"body":null,"footers":[]}"#
        );
    }

    #[test]
    fn test_render_json_valid_full() {
        let outcome = evaluate_default("feat(api)!: add \"v2\" endpoint\n\nNew body\n\nRefs: #12");
        assert_eq!(
            render_json(&outcome),
            r#"{"valid":true,"type":"feat","scope":"api","description":"add \"v2\" endpoint","breaking_change":true,"body":"New body","footers":["Refs: #12"]}"#
        );
    }

    #[test]
    fn test_render_json_invalid_lists_every_error() {
        let config = ValidationConfig {
            require_scope: true,
            min_description_length: 50,
            ..ValidationConfig::default()
        };
        let outcome = evaluate("feat: short", &config);
        let json = render_json(&outcome);

        assert!(json.starts_with(r#"{"valid":false,"errors":["#), "{json}");
        let Outcome::Invalid(error) = &outcome else {
            panic!("expected invalid outcome");
        };
        let messages = error_messages(error);
        assert!(
            messages.len() >= 2,
            "expected multiple errors: {messages:?}"
        );
        for message in messages {
            assert!(json.contains(&json_escape(&message)), "{json}");
        }
    }

    #[test]
    fn test_error_messages_single_variant() {
        let Outcome::Invalid(error) = evaluate_default("garbage") else {
            panic!("expected invalid outcome");
        };
        assert_eq!(error_messages(&error), vec![error.to_string()]);
    }

    #[test]
    fn test_render_text_missing() {
        let (out, err) = render_text_to_strings(&Outcome::Missing, "", false);
        assert!(out.is_empty());
        assert_eq!(err, "Error: No commit message provided\n");
    }

    #[test]
    fn test_render_text_valid_not_verbose() {
        let outcome = evaluate_default("feat: add thing");
        let (out, err) = render_text_to_strings(&outcome, "feat: add thing", false);
        assert_eq!(
            out,
            "✅ Valid conventional commit!\n✅ Valid conventional commit\n"
        );
        assert!(err.is_empty());
    }

    #[test]
    fn test_render_text_valid_verbose() {
        let message = "feat(auth)!: add login\n\nSome body\n\nRefs: #1";
        let outcome = evaluate_default(message);
        let (out, err) = render_text_to_strings(&outcome, message, true);

        assert!(err.is_empty());
        assert!(out.contains("Type: feat\n"), "{out}");
        assert!(out.contains("Scope: auth\n"), "{out}");
        assert!(out.contains("Description: add login\n"), "{out}");
        assert!(out.contains("BREAKING CHANGE detected!"), "{out}");
        assert!(out.contains("Body: Some body\n"), "{out}");
        assert!(out.contains(r#"Footers: ["Refs: #1"]"#), "{out}");
        assert!(!out.contains("✅ Valid conventional commit\n✅"), "{out}");
    }

    #[test]
    fn test_render_text_valid_verbose_omits_absent_parts() {
        let outcome = evaluate_default("fix: resolve bug");
        let (out, _) = render_text_to_strings(&outcome, "fix: resolve bug", true);

        assert!(!out.contains("Scope:"), "{out}");
        assert!(!out.contains("BREAKING"), "{out}");
        assert!(!out.contains("Body:"), "{out}");
        assert!(!out.contains("Footers:"), "{out}");
    }

    #[test]
    fn test_render_text_invalid_not_verbose() {
        let outcome = evaluate_default("garbage");
        let (out, err) = render_text_to_strings(&outcome, "garbage", false);

        assert!(out.is_empty());
        assert!(err.starts_with("❌ Invalid commit message: "), "{err}");
        assert!(!err.contains("Message was:"), "{err}");
    }

    #[test]
    fn test_render_text_invalid_verbose() {
        let outcome = evaluate_default("garbage");
        let (out, err) = render_text_to_strings(&outcome, "garbage", true);

        assert!(err.contains("Message was: 'garbage'\n"), "{err}");
        assert_eq!(out, HELP_HINT);
    }
}
