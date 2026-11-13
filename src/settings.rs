//! Merges CLI arguments, the `.commit-check.toml` file and hardcoded defaults
//! into the effective settings used by the binary.
//!
//! Precedence for scalar options: CLI flag > config file > default.
//! List options (`clean_starts_with`, `clean_regex`) are concatenated: CLI values first,
//! then config file values.

use conventional_commits::ValidationConfig;

use crate::cli::Cli;
use crate::config::FileConfig;

/// Rules used to strip lines from the commit body before validation.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct CleanRules {
    /// Lines starting with any of these prefixes are removed.
    pub(crate) starts_with: Vec<String>,
    /// Lines matching any of these regex patterns are removed.
    pub(crate) regexes: Vec<String>,
}

/// Effective settings after merging every configuration source.
#[derive(Debug)]
pub(crate) struct Settings {
    pub(crate) validation: ValidationConfig,
    pub(crate) clean: CleanRules,
}

/// Parses a comma-separated string into a list of non-empty trimmed strings.
fn split_csv(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

/// Concatenates CLI values with the comma-separated values from the config file.
fn merge_list(cli_values: &[String], file_csv: Option<&str>) -> Vec<String> {
    let mut merged = cli_values.to_vec();
    if let Some(csv) = file_csv {
        merged.extend(split_csv(csv));
    }
    merged
}

/// Builds the effective [`Settings`] from CLI arguments and the config file.
pub(crate) fn resolve(cli: &Cli, file_cfg: &FileConfig) -> Settings {
    let defaults = ValidationConfig::default();

    let allowed_scopes = cli
        .scopes
        .as_deref()
        .or(file_cfg.scopes.as_deref())
        .map(split_csv);

    let validation = ValidationConfig {
        max_description_length: cli
            .max_length
            .or(file_cfg.max_length)
            .unwrap_or(defaults.max_description_length),
        min_description_length: cli
            .min_length
            .or(file_cfg.min_length)
            .unwrap_or(defaults.min_description_length),
        allow_custom_types: cli
            .allow_custom_types
            .or(file_cfg.allow_custom_types)
            .unwrap_or(defaults.allow_custom_types),
        enforce_lowercase_description: cli
            .enforce_lowercase
            .or(file_cfg.enforce_lowercase)
            .unwrap_or(defaults.enforce_lowercase_description),
        enforce_lowercase_scope: cli
            .enforce_lowercase_scope
            .or(file_cfg.enforce_lowercase_scope)
            .unwrap_or(defaults.enforce_lowercase_scope),
        require_scope: cli
            .require_scope
            .or(file_cfg.require_scope)
            .unwrap_or(defaults.require_scope),
        disallow_description_period: cli
            .disallow_period
            .or(file_cfg.disallow_period)
            .unwrap_or(defaults.disallow_description_period),
        allowed_scopes,
        issue_pattern: cli
            .issue_pattern
            .clone()
            .or_else(|| file_cfg.issue_pattern.clone()),
        allowed_types: cli.types.clone(),
    };

    let clean = CleanRules {
        starts_with: merge_list(
            &cli.clean_starts_with,
            file_cfg.clean_starts_with.as_deref(),
        ),
        regexes: merge_list(&cli.clean_regex, file_cfg.clean_regex.as_deref()),
    };

    Settings { validation, clean }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse_cli(args: &[&str]) -> Cli {
        let mut argv = vec!["commit-check"];
        argv.extend_from_slice(args);
        Cli::try_parse_from(argv).expect("valid CLI arguments")
    }

    fn full_file_config() -> FileConfig {
        FileConfig {
            allow_custom_types: Some(false),
            max_length: Some(50),
            min_length: Some(5),
            enforce_lowercase: Some(false),
            disallow_period: Some(false),
            enforce_lowercase_scope: Some(true),
            require_scope: Some(true),
            scopes: Some("api, web".to_string()),
            issue_pattern: Some("^Refs: #\\d+".to_string()),
            clean_starts_with: Some("Signed-off-by".to_string()),
            clean_regex: Some("^Change-Id:.*".to_string()),
        }
    }

    #[test]
    fn test_split_csv_trims_and_skips_empty() {
        assert_eq!(split_csv(" a, b ,,c ,"), vec!["a", "b", "c"]);
        assert!(split_csv("").is_empty());
        assert!(split_csv(" , ").is_empty());
    }

    #[test]
    fn test_merge_list_cli_first_then_file() {
        let cli_values = vec!["x".to_string()];
        assert_eq!(merge_list(&cli_values, Some("y, z")), vec!["x", "y", "z"]);
        assert_eq!(merge_list(&cli_values, None), vec!["x"]);
        assert!(merge_list(&[], None).is_empty());
    }

    #[test]
    fn test_resolve_defaults_when_nothing_set() {
        let settings = resolve(&parse_cli(&[]), &FileConfig::default());
        let validation = settings.validation;
        let defaults = ValidationConfig::default();

        assert_eq!(validation.max_description_length, 72);
        assert_eq!(validation.min_description_length, 0);
        assert_eq!(validation.allow_custom_types, defaults.allow_custom_types);
        assert!(validation.enforce_lowercase_description);
        assert!(!validation.enforce_lowercase_scope);
        assert!(!validation.require_scope);
        assert!(validation.disallow_description_period);
        assert_eq!(validation.allowed_scopes, None);
        assert_eq!(validation.issue_pattern, None);
        assert_eq!(validation.allowed_types, None);
        assert_eq!(settings.clean, CleanRules::default());
    }

    #[test]
    fn test_resolve_uses_file_values_when_cli_absent() {
        let settings = resolve(&parse_cli(&[]), &full_file_config());
        let validation = settings.validation;

        assert_eq!(validation.max_description_length, 50);
        assert_eq!(validation.min_description_length, 5);
        assert!(!validation.allow_custom_types);
        assert!(!validation.enforce_lowercase_description);
        assert!(validation.enforce_lowercase_scope);
        assert!(validation.require_scope);
        assert!(!validation.disallow_description_period);
        assert_eq!(
            validation.allowed_scopes,
            Some(vec!["api".to_string(), "web".to_string()])
        );
        assert_eq!(validation.issue_pattern.as_deref(), Some("^Refs: #\\d+"));
        assert_eq!(settings.clean.starts_with, vec!["Signed-off-by"]);
        assert_eq!(settings.clean.regexes, vec!["^Change-Id:.*"]);
    }

    #[test]
    fn test_resolve_cli_overrides_file() {
        let cli = parse_cli(&[
            "--max-length",
            "100",
            "--min-length",
            "1",
            "--allow-custom-types",
            "true",
            "--enforce-lowercase",
            "true",
            "--enforce-lowercase-scope",
            "false",
            "--require-scope",
            "false",
            "--disallow-period",
            "true",
            "--scopes",
            "core",
            "--issue-pattern",
            "^Fixes: .+",
        ]);
        let validation = resolve(&cli, &full_file_config()).validation;

        assert_eq!(validation.max_description_length, 100);
        assert_eq!(validation.min_description_length, 1);
        assert!(validation.allow_custom_types);
        assert!(validation.enforce_lowercase_description);
        assert!(!validation.enforce_lowercase_scope);
        assert!(!validation.require_scope);
        assert!(validation.disallow_description_period);
        assert_eq!(validation.allowed_scopes, Some(vec!["core".to_string()]));
        assert_eq!(validation.issue_pattern.as_deref(), Some("^Fixes: .+"));
    }

    #[test]
    fn test_resolve_clean_rules_concatenate_cli_and_file() {
        let cli = parse_cli(&[
            "--clean-starts-with",
            "Co-authored-by",
            "--clean-regex",
            "^Reviewed-on:.*",
        ]);
        let clean = resolve(&cli, &full_file_config()).clean;

        assert_eq!(clean.starts_with, vec!["Co-authored-by", "Signed-off-by"]);
        assert_eq!(clean.regexes, vec!["^Reviewed-on:.*", "^Change-Id:.*"]);
    }

    #[test]
    fn test_resolve_types_from_cli() {
        let cli = parse_cli(&["--types", "feat,fix"]);
        let validation = resolve(&cli, &FileConfig::default()).validation;

        assert_eq!(
            validation.allowed_types,
            Some(vec!["feat".to_string(), "fix".to_string()])
        );
    }

    #[test]
    fn test_resolve_scopes_csv_ignores_empty_entries() {
        let cli = parse_cli(&["--scopes", " api , ,web,"]);
        let validation = resolve(&cli, &FileConfig::default()).validation;

        assert_eq!(
            validation.allowed_scopes,
            Some(vec!["api".to_string(), "web".to_string()])
        );
    }
}
