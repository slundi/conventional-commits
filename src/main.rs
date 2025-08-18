use clap::{Args, Parser, Subcommand};
use conventional_commits::{validate_commit_with_config, ValidationConfig};
use std::io::{self, Read};
use std::process;

#[derive(Parser)]
#[command(name = "commit-check")]
#[command(about = "A CLI tool for validating Conventional Commits")]
#[command(version = "0.1.0")]
#[command(author = "Your Name <your.email@example.com>")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Commit message to validate (if not provided, reads from stdin)
    #[arg(short, long)]
    message: Option<String>,

    /// Allow custom commit types beyond the standard ones
    #[arg(long, default_value = "true")]
    allow_custom_types: bool,

    /// Maximum length for commit description
    #[arg(long, default_value = "72")]
    max_length: usize,

    /// Enforce lowercase description
    #[arg(long, default_value = "true")]
    enforce_lowercase: bool,

    /// Disallow period at end of description
    #[arg(long, default_value = "true")]
    disallow_period: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a commit message
    Validate(ValidateArgs),
    /// Show examples of valid commit messages
    Examples,
    /// Show commit types information
    Types,
}

#[derive(Args)]
struct ValidateArgs {
    /// Commit message to validate
    message: Option<String>,
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

fn show_examples() {
    println!("Examples of valid Conventional Commits:");
    println!();
    println!("Basic commits:");
    println!("  feat: add user registration");
    println!("  fix: resolve memory leak in parser");
    println!("  docs: update installation guide");
    println!("  style: format code with prettier");
    println!("  refactor: extract validation logic");
    println!("  test: add unit tests for auth module");
    println!("  chore: update dependencies");
    println!();
    println!("With scope:");
    println!("  feat(auth): implement OAuth2 flow");
    println!("  fix(parser): handle edge case in tokenizer");
    println!("  docs(api): add endpoint documentation");
    println!();
    println!("Breaking changes:");
    println!("  feat!: change API response format");
    println!("  feat: add new feature");
    println!("  ");
    println!("  BREAKING CHANGE: API response format has changed");
    println!();
    println!("With body and footer:");
    println!("  fix: prevent racing of requests");
    println!("  ");
    println!("  Introduce a request id and a reference to latest request. Dismiss");
    println!("  incoming responses other than from latest request.");
    println!("  ");
    println!("  Remove timeouts which were used to mitigate the racing issue but are");
    println!("  obsolete now.");
    println!("  ");
    println!("  Reviewed-by: Z");
    println!("  Refs: #123");
}

fn show_types() {
    println!("Standard Conventional Commit types:");
    println!();
    println!("  feat     - A new feature");
    println!("  fix      - A bug fix");
    println!("  docs     - Documentation only changes");
    println!("  style    - Changes that do not affect the meaning of the code");
    println!("  refactor - A code change that neither fixes a bug nor adds a feature");
    println!("  perf     - A code change that improves performance");
    println!("  test     - Adding missing tests or correcting existing tests");
    println!("  build    - Changes to the build process or auxiliary tools");
    println!("  ci       - Changes to CI configuration files and scripts");
    println!("  chore    - Other changes that don't modify src or test files");
    println!("  revert   - Reverts a previous commit");
    println!();
    println!("Custom types are also allowed by default.");
    println!("Use --allow-custom-types=false to restrict to standard types only.");
}

fn show_help_hint() {
    println!();
    println!("For examples of valid commits, run: commit-check examples");
    println!("For information about commit types, run: commit-check types");
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn test_valid_commit_via_arg() {
        let mut cmd = Command::cargo_bin("commit-check").unwrap();
        cmd.arg("--message")
            .arg("feat: add new feature")
            .assert()
            .success()
            .stdout(predicate::str::contains("Valid conventional commit"));
    }

    #[test]
    fn test_invalid_commit_via_arg() {
        let mut cmd = Command::cargo_bin("commit-check").unwrap();
        cmd.arg("--message")
            .arg("invalid commit")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Invalid commit message"));
    }

    #[test]
    fn test_examples_command() {
        let mut cmd = Command::cargo_bin("commit-check").unwrap();
        cmd.arg("examples")
            .assert()
            .success()
            .stdout(predicate::str::contains("Examples of valid Conventional Commits"));
    }

    #[test]
    fn test_types_command() {
        let mut cmd = Command::cargo_bin("commit-check").unwrap();
        cmd.arg("types")
            .assert()
            .success()
            .stdout(predicate::str::contains("Standard Conventional Commit types"));
    }
}
