mod cli;
mod config;
mod report;
mod settings;

use clap::Parser;
use conventional_commits::{ValidationConfig, clean_commit_body, fix_commit_message};
use std::io::{self, Read, Write};
use std::process;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::cli::{Cli, Commands, OutputFormat, show_examples, show_types};

fn install_git_hook(
    base_dir: &std::path::Path,
    force: bool,
    extra_args: Option<&str>,
) -> std::io::Result<()> {
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

    let extra = extra_args
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!(" {}", s))
        .unwrap_or_default();

    // The script reads the commit message file ($1) and pipes it to commit-check
    let hook_content = format!(
        "#!/bin/sh\n# commit-check git hook\ncommit-check validate --message \"$(cat \"$1\")\" --verbose{}\n",
        extra
    );

    fs::write(&hook_path, &hook_content)?;

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

/// Applies clean rules (starts_with + regex) when any rules are configured.
/// Prints removed lines and returns the cleaned message.
fn apply_clean_rules(message: String, starts_with: &[String], regexes: &[String]) -> String {
    if starts_with.is_empty() && regexes.is_empty() {
        return message;
    }
    let sw_refs: Vec<&str> = starts_with.iter().map(String::as_str).collect();
    let re_refs: Vec<&str> = regexes.iter().map(String::as_str).collect();
    match clean_commit_body(&message, &sw_refs, &re_refs) {
        Err(e) => {
            eprintln!("Warning: invalid clean-regex pattern: {e}");
            message
        }
        Ok(result) => {
            for line in &result.removed_lines {
                print_color("Removed: ", Color::Yellow, true, false);
                println!("{}", line);
            }
            result.cleaned_message
        }
    }
}

fn apply_fix_if_requested(message: String, fix: bool) -> String {
    if !fix {
        return message;
    }
    let fixed = fix_commit_message(&message);
    if fixed != message {
        print_color("Fixed: ", Color::Cyan, true, false);
    }
    // Always print when --fix is active so the result is capturable by scripts
    println!("{}", fixed);
    fixed
}

fn main() {
    let cli = Cli::parse();

    let file_cfg = match config::load_config() {
        Ok(cfg) => cfg.unwrap_or_default(),
        Err(e) => {
            eprintln!("Warning: {e}");
            config::FileConfig::default()
        }
    };

    let settings::Settings {
        validation: config,
        clean,
    } = settings::resolve(&cli, &file_cfg);

    let format = cli.format.clone();

    match &cli.command {
        Some(Commands::Validate(args)) => {
            let message = get_commit_message(args.message.as_deref(), cli.message.as_deref());
            let message = apply_clean_rules(message, &clean.starts_with, &clean.regexes);
            let message = apply_fix_if_requested(message, cli.fix);
            exit_on_failure(validate_and_report(&message, &config, cli.verbose, &format));
        }
        Some(Commands::Examples) => show_examples(),
        Some(Commands::Types) => show_types(),
        Some(Commands::InstallHook(args)) => {
            let base_dir = std::path::Path::new(".");
            if let Err(e) = install_git_hook(base_dir, args.force, args.hook_args.as_deref()) {
                eprintln!("❌ Failed to install hook: {}", e);
                process::exit(1);
            }
            println!("✅ Git hook installed successfully at .git/hooks/commit-msg");
        }
        None => {
            // Default behavior: validate commit message
            let message = get_commit_message(None, cli.message.as_deref());
            let message = apply_clean_rules(message, &clean.starts_with, &clean.regexes);
            let message = apply_fix_if_requested(message, cli.fix);
            exit_on_failure(validate_and_report(&message, &config, cli.verbose, &format));
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

/// Validates the message, prints the result in the requested format and
/// returns the process exit code.
fn validate_and_report(
    message: &str,
    config: &ValidationConfig,
    verbose: bool,
    format: &OutputFormat,
) -> i32 {
    let outcome = report::evaluate(message, config);
    match format {
        OutputFormat::Json => println!("{}", report::render_json(&outcome)),
        OutputFormat::Text => {
            let stdout = StandardStream::stdout(ColorChoice::Auto);
            let stderr = StandardStream::stderr(ColorChoice::Auto);
            let rendered = report::render_text(
                &outcome,
                message,
                verbose,
                &mut stdout.lock(),
                &mut stderr.lock(),
            );
            if let Err(error) = rendered {
                eprintln!("Error writing output: {error}");
            }
        }
    }
    outcome.exit_code()
}

/// Exits the process when `code` signals a failure.
fn exit_on_failure(code: i32) {
    if code != 0 {
        process::exit(code);
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
        let result = install_git_hook(&tmp, false, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_install_hook_success() {
        let tmp = make_tmp_dir();
        std::fs::create_dir_all(tmp.join(".git/hooks")).unwrap();

        let result = install_git_hook(&tmp, false, None);
        assert!(result.is_ok());
        assert!(tmp.join(".git/hooks/commit-msg").exists());

        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_install_hook_existing_without_force() {
        let tmp = make_tmp_dir();
        std::fs::create_dir_all(tmp.join(".git/hooks")).unwrap();
        std::fs::write(tmp.join(".git/hooks/commit-msg"), "existing hook").unwrap();

        let result = install_git_hook(&tmp, false, None);
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

        let result = install_git_hook(&tmp, true, None);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(tmp.join(".git/hooks/commit-msg")).unwrap();
        assert!(content.contains("commit-check"));

        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_install_hook_with_extra_args() {
        let tmp = make_tmp_dir();
        std::fs::create_dir_all(tmp.join(".git/hooks")).unwrap();

        let result = install_git_hook(&tmp, false, Some("--scopes api,client --min-length 10"));
        assert!(result.is_ok());

        let content = std::fs::read_to_string(tmp.join(".git/hooks/commit-msg")).unwrap();
        assert!(
            content.contains("--scopes api,client --min-length 10"),
            "hook should embed extra args: {content}"
        );

        std::fs::remove_dir_all(&tmp).unwrap();
    }
}
