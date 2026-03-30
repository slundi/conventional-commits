mod cli;

use clap::Parser;
use conventional_commits::{
    fix_commit_message, validate_commit_with_config, CommitValidationError, ValidationConfig,
};
use std::io::{self, Read, Write};
use std::process;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::cli::{show_examples, show_help_hint, show_types, Cli, Commands, OutputFormat};

fn install_git_hook(base_dir: &std::path::Path, force: bool) -> std::io::Result<()> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt; // Note: For Linux/macOS

    let hook_dir = base_dir.join(".git/hooks");
    if !hook_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Not a git repository (no .git/hooks found)",
        ));
    }

    let hook_path = hook_dir.join("commit-msg");

    if hook_path.exists() && !force {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            ".git/hooks/commit-msg already exists. Use --force to overwrite.",
        ));
    }

    // The script reads the commit message file ($1) and pipes it to commit-check
    let hook_content = r#"#!/bin/sh
# commit-check git hook
commit-check validate --message "$(cat "$1")" --verbose
"#;

    fs::write(&hook_path, hook_content)?;

    // Make the script executable (Unix-specific)
    let mut perms = fs::metadata(&hook_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms)?;

    Ok(())
}

/// Helper for colored console output
fn print_color(message: &str, color: Color, is_bold: bool, is_stderr: bool) {
    let mut stream = if is_stderr {
        StandardStream::stderr(ColorChoice::Auto)
    } else {
        StandardStream::stdout(ColorChoice::Auto)
    };

    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color)).set_bold(is_bold);

    let _ = stream.set_color(&spec);
    let _ = write!(&mut stream, "{}", message);
    let _ = stream.reset();
}

fn apply_fix_if_requested(message: String, fix: bool) -> String {
    if !fix {
        return message;
    }
    let fixed = fix_commit_message(&message);
    if fixed != message {
        print_color("Fixed: ", Color::Cyan, true, false);
        println!("{}", fixed);
    }
    fixed
}

fn main() {
    let cli = Cli::parse();

    let allowed_scopes = cli.scopes.as_deref().map(|s| {
        s.split(',')
            .map(|scope| scope.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });

    let config = ValidationConfig {
        max_description_length: cli.max_length,
        min_description_length: cli.min_length,
        allow_custom_types: cli.allow_custom_types,
        enforce_lowercase_description: cli.enforce_lowercase,
        disallow_description_period: cli.disallow_period,
        allowed_scopes,
        issue_pattern: cli.issue_pattern,
    };

    let format = cli.format.clone();

    match &cli.command {
        Some(Commands::Validate(args)) => {
            let message = get_commit_message(args.message.as_deref(), cli.message.as_deref());
            let message = apply_fix_if_requested(message, cli.fix);
            validate_and_report(&message, &config, cli.verbose, &format);
        }
        Some(Commands::Examples) => show_examples(),
        Some(Commands::Types) => show_types(),
        Some(Commands::InstallHook(args)) => {
            let base_dir = std::path::Path::new(".");
            if let Err(e) = install_git_hook(base_dir, args.force) {
                eprintln!("❌ Failed to install hook: {}", e);
                process::exit(1);
            }
            println!("✅ Git hook installed successfully at .git/hooks/commit-msg");
        }
        None => {
            // Default behavior: validate commit message
            let message = get_commit_message(None, cli.message.as_deref());
            let message = apply_fix_if_requested(message, cli.fix);
            validate_and_report(&message, &config, cli.verbose, &format);
        }
    }
}

fn get_commit_message(arg_message: Option<&str>, cli_message: Option<&str>) -> String {
    if let Some(msg) = arg_message.or(cli_message) {
        return msg.to_string();
    }

    // Read from stdin
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Ok(_) => buffer.trim().to_string(),
        Err(e) => {
            eprintln!("Error reading from stdin: {}", e);
            process::exit(1);
        }
    }
}

/// Escapes a string for safe embedding in a JSON value.
fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Flattens a `CommitValidationError` into individual error message strings.
/// `Multiple` is unpacked; all other variants become a single-element vec.
fn error_messages(e: &CommitValidationError) -> Vec<String> {
    match e {
        CommitValidationError::Multiple(errs) => errs.0.iter().map(|e| e.to_string()).collect(),
        _ => vec![e.to_string()],
    }
}

fn validate_and_report(
    message: &str,
    config: &ValidationConfig,
    verbose: bool,
    format: &OutputFormat,
) {
    if message.is_empty() {
        if *format == OutputFormat::Json {
            println!(r#"{{"valid":false,"errors":["No commit message provided"]}}"#);
        } else {
            eprintln!("Error: No commit message provided");
        }
        process::exit(1);
    }

    match validate_commit_with_config(message, config) {
        Ok(commit) => {
            if *format == OutputFormat::Json {
                let scope_json = match &commit.scope {
                    Some(s) => format!("\"{}\"", json_escape(s)),
                    None => "null".to_string(),
                };
                let body_json = match &commit.body {
                    Some(b) => format!("\"{}\"", json_escape(b)),
                    None => "null".to_string(),
                };
                let footers_json = commit
                    .footers
                    .iter()
                    .map(|f| format!("\"{}\"", json_escape(f)))
                    .collect::<Vec<_>>()
                    .join(",");
                println!(
                    r#"{{"valid":true,"type":"{}","scope":{},"description":"{}","breaking_change":{},"body":{},"footers":[{}]}}"#,
                    json_escape(&commit.commit_type.to_string()),
                    scope_json,
                    json_escape(&commit.description),
                    commit.is_breaking_change(),
                    body_json,
                    footers_json,
                );
                return;
            }

            print_color("✅ Valid conventional commit!", Color::Green, true, false);
            println!();

            if verbose {
                println!("Type: {}", commit.commit_type);
                if let Some(scope) = &commit.scope {
                    println!("Scope: {}", scope);
                }
                println!("Description: {}", commit.description);
                if commit.is_breaking_change() {
                    print_color("⚠️  BREAKING CHANGE detected!", Color::Yellow, true, false);
                    println!();
                }
                if let Some(body) = &commit.body {
                    println!("Body: {}", body);
                }
                if !commit.footers.is_empty() {
                    println!("Footers: {:?}", commit.footers);
                }
            } else {
                println!("✅ Valid conventional commit");
            }
        }
        Err(e) => {
            if *format == OutputFormat::Json {
                let errors_json = error_messages(&e)
                    .iter()
                    .map(|m| format!("\"{}\"", json_escape(m)))
                    .collect::<Vec<_>>()
                    .join(",");
                println!(r#"{{"valid":false,"errors":[{}]}}"#, errors_json);
                process::exit(1);
            }

            print_color("❌ Invalid commit message: ", Color::Red, true, true);
            eprintln!("{}", e);
            if verbose {
                eprintln!("Message was: '{}'", message);
                show_help_hint();
            }
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn make_tmp_dir() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("cc_hook_test_{}_{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_get_commit_message_from_arg() {
        let message = get_commit_message(Some("test message"), None);
        assert_eq!(message, "test message");
    }

    #[test]
    fn test_get_commit_message_from_cli() {
        let message = get_commit_message(None, Some("cli message"));
        assert_eq!(message, "cli message");
    }

    #[test]
    fn test_get_commit_message_precedence() {
        let message = get_commit_message(Some("arg message"), Some("cli message"));
        assert_eq!(message, "arg message");
    }

    #[test]
    fn test_install_hook_no_git_dir() {
        let tmp = make_tmp_dir();
        let result = install_git_hook(&tmp, false);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_install_hook_success() {
        let tmp = make_tmp_dir();
        std::fs::create_dir_all(tmp.join(".git/hooks")).unwrap();

        let result = install_git_hook(&tmp, false);
        assert!(result.is_ok());
        assert!(tmp.join(".git/hooks/commit-msg").exists());

        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_install_hook_existing_without_force() {
        let tmp = make_tmp_dir();
        std::fs::create_dir_all(tmp.join(".git/hooks")).unwrap();
        std::fs::write(tmp.join(".git/hooks/commit-msg"), "existing hook").unwrap();

        let result = install_git_hook(&tmp, false);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::AlreadyExists
        );

        // Original hook must be untouched
        let content = std::fs::read_to_string(tmp.join(".git/hooks/commit-msg")).unwrap();
        assert_eq!(content, "existing hook");

        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_install_hook_existing_with_force() {
        let tmp = make_tmp_dir();
        std::fs::create_dir_all(tmp.join(".git/hooks")).unwrap();
        std::fs::write(tmp.join(".git/hooks/commit-msg"), "existing hook").unwrap();

        let result = install_git_hook(&tmp, true);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(tmp.join(".git/hooks/commit-msg")).unwrap();
        assert!(content.contains("commit-check"));

        std::fs::remove_dir_all(&tmp).unwrap();
    }
}
