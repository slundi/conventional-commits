mod cli;

use clap::Parser;
use conventional_commits::{validate_commit_with_config, ValidationConfig};
use std::io::{self, Read};
use std::process;

use crate::cli::{show_examples, show_help_hint, show_types, Cli, Commands};

fn install_git_hook() -> std::io::Result<()> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt; // Note: For Linux/macOS

    let hook_dir = std::path::Path::new(".git/hooks");
    if !hook_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Not a git repository (no .git/hooks found)",
        ));
    }

    let hook_path = hook_dir.join("commit-msg");

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

fn main() {
    let cli = Cli::parse();

    let config = ValidationConfig {
        max_description_length: cli.max_length,
        allow_custom_types: cli.allow_custom_types,
        enforce_lowercase_description: cli.enforce_lowercase,
        disallow_description_period: cli.disallow_period,
    };

    match &cli.command {
        Some(Commands::Validate(args)) => {
            let message = get_commit_message(args.message.as_deref(), cli.message.as_deref());
            validate_and_report(&message, &config, cli.verbose);
        }
        Some(Commands::Examples) => show_examples(),
        Some(Commands::Types) => show_types(),
        Some(Commands::InstallHook) => {
            if let Err(e) = install_git_hook() {
                eprintln!("❌ Failed to install hook: {}", e);
                process::exit(1);
            }
            println!("✅ Git hook installed successfully at .git/hooks/commit-msg");
        }
        None => {
            // Default behavior: validate commit message
            let message = get_commit_message(None, cli.message.as_deref());
            validate_and_report(&message, &config, cli.verbose);
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

fn validate_and_report(message: &str, config: &ValidationConfig, verbose: bool) {
    if message.is_empty() {
        eprintln!("Error: No commit message provided");
        process::exit(1);
    }

    match validate_commit_with_config(message, config) {
        Ok(commit) => {
            if verbose {
                println!("✅ Valid conventional commit!");
                println!("Type: {}", commit.commit_type);
                if let Some(scope) = &commit.scope {
                    println!("Scope: {}", scope);
                }
                println!("Description: {}", commit.description);
                if commit.is_breaking_change() {
                    println!("⚠️  Breaking change detected!");
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
            eprintln!("❌ Invalid commit message: {}", e);
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
}
