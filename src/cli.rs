use clap::{Args, Parser, Subcommand, ValueEnum};

/// Output format for validation results
#[derive(ValueEnum, Clone, Default, PartialEq, Debug)]
pub(crate) enum OutputFormat {
    /// Human-readable coloured text (default)
    #[default]
    Text,
    /// Machine-readable JSON
    Json,
}

#[derive(Parser)]
#[command(name = "commit-check")]
#[command(about = "A CLI tool for validating Conventional Commits")]
#[command(version = "0.1.0")]
#[command(author = "Your Name <your.email@example.com>")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,

    /// Commit message to validate (if not provided, reads from stdin)
    #[arg(short, long)]
    pub(crate) message: Option<String>,

    /// Allow custom commit types beyond the standard ones [default: true]
    #[arg(long, value_name = "BOOL")]
    pub(crate) allow_custom_types: Option<bool>,

    /// Maximum length for commit description [default: 72]
    #[arg(long, value_name = "N")]
    pub(crate) max_length: Option<usize>,

    /// Minimum length for commit description [default: 0]
    #[arg(long, value_name = "N")]
    pub(crate) min_length: Option<usize>,

    /// Enforce lowercase description [default: true]
    #[arg(long, value_name = "BOOL")]
    pub(crate) enforce_lowercase: Option<bool>,

    /// Disallow period at end of description [default: true]
    #[arg(long, value_name = "BOOL")]
    pub(crate) disallow_period: Option<bool>,

    /// Enable verbose output
    #[arg(short, long)]
    pub(crate) verbose: bool,

    /// Automatically fix common mistakes (missing space, uppercase first letter, trailing period)
    /// and print the corrected message
    #[arg(long)]
    pub(crate) fix: bool,

    /// Comma-separated list of allowed scopes (e.g. "api,client,shared").
    /// When set, commits with a scope not in this list are rejected.
    #[arg(long, value_name = "SCOPES")]
    pub(crate) scopes: Option<String>,

    /// Enforce lowercase scope [default: false]
    #[arg(long, value_name = "BOOL")]
    pub(crate) enforce_lowercase_scope: Option<bool>,

    /// Require a scope to be present on every commit [default: false]
    #[arg(long, value_name = "BOOL")]
    pub(crate) require_scope: Option<bool>,

    /// Regex pattern that at least one footer must match (e.g. "^Refs: #\d+" or "^Fixes: [A-Z]+-\d+").
    /// Ensures commits are linked to an issue or ticket.
    #[arg(long, value_name = "PATTERN")]
    pub(crate) issue_pattern: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    pub(crate) format: OutputFormat,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Validate a commit message
    Validate(ValidateArgs),
    /// Show examples of valid commit messages
    Examples,
    /// Show commit types information
    Types,
    /// Install as a git commit-msg hook
    InstallHook(InstallHookArgs),
}

#[derive(Args)]
pub(crate) struct InstallHookArgs {
    /// Overwrite an existing hook without prompting
    #[arg(long)]
    pub(crate) force: bool,

    /// Extra arguments appended to the commit-check call inside the hook
    /// (e.g. "--scopes api,client --issue-pattern '^Refs: #\\d+'")
    #[arg(long, value_name = "ARGS")]
    pub(crate) hook_args: Option<String>,
}

#[derive(Args)]
pub(crate) struct ValidateArgs {
    /// Commit message to validate
    pub(crate) message: Option<String>,
}

pub(crate) fn show_examples() {
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

pub(crate) fn show_types() {
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

pub(crate) fn show_help_hint() {
    println!();
    println!("For examples of valid commits, run: commit-check examples");
    println!("For information about commit types, run: commit-check types");
}
