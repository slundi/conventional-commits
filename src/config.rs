use std::path::{Path, PathBuf};
use thiserror::Error;

const CONFIG_FILE_NAME: &str = ".commit-check.toml";

/// Configuration loaded from a `.commit-check.toml` file.
/// All fields are optional — absent fields fall back to CLI args, then hardcoded defaults.
#[derive(Debug, Default)]
pub(crate) struct FileConfig {
    pub(crate) allow_custom_types: Option<bool>,
    pub(crate) max_length: Option<usize>,
    pub(crate) min_length: Option<usize>,
    pub(crate) enforce_lowercase: Option<bool>,
    pub(crate) disallow_period: Option<bool>,
    pub(crate) enforce_lowercase_scope: Option<bool>,
    pub(crate) require_scope: Option<bool>,
    pub(crate) scopes: Option<String>,
    pub(crate) issue_pattern: Option<String>,
}

/// Errors that can occur when loading and parsing a config file.
#[derive(Error, Debug)]
pub(crate) enum TomlError {
    #[error("cannot read '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot parse '{path}': {message}")]
    Parse { path: PathBuf, message: String },
}

/// Searches for `.commit-check.toml` starting from `dir`, walking up to the filesystem root.
pub(crate) fn find_config_file(dir: &Path) -> Option<PathBuf> {
    let mut current = dir.to_path_buf();
    loop {
        let candidate = current.join(CONFIG_FILE_NAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Reads and parses a `.commit-check.toml` at the given path.
pub(crate) fn load_config_from_path(path: &Path) -> Result<FileConfig, TomlError> {
    let content = std::fs::read_to_string(path).map_err(|e| TomlError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    parse_config(&content).map_err(|e| TomlError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

/// Searches for a `.commit-check.toml` starting from the current directory and walking up.
/// Returns `Ok(None)` when no config file is found.
pub(crate) fn load_config() -> Result<Option<FileConfig>, TomlError> {
    let cwd = std::env::current_dir().map_err(|e| TomlError::Io {
        path: PathBuf::from("."),
        source: e,
    })?;
    match find_config_file(&cwd) {
        Some(path) => load_config_from_path(&path).map(Some),
        None => Ok(None),
    }
}

/// Parses a TOML string into a `FileConfig`.
fn parse_config(content: &str) -> Result<FileConfig, toml_span::DeserError> {
    use toml_span::de_helpers::TableHelper;

    let mut val = toml_span::parse(content)?;
    let mut th = TableHelper::new(&mut val)?;

    let allow_custom_types = th.optional::<bool>("allow_custom_types");
    let max_length = th.optional::<usize>("max_length");
    let min_length = th.optional::<usize>("min_length");
    let enforce_lowercase = th.optional::<bool>("enforce_lowercase");
    let disallow_period = th.optional::<bool>("disallow_period");
    let enforce_lowercase_scope = th.optional::<bool>("enforce_lowercase_scope");
    let require_scope = th.optional::<bool>("require_scope");
    let scopes = th.optional::<String>("scopes");
    let issue_pattern = th.optional::<String>("issue_pattern");

    th.finalize(None)?;

    Ok(FileConfig {
        allow_custom_types,
        max_length,
        min_length,
        enforce_lowercase,
        disallow_period,
        enforce_lowercase_scope,
        require_scope,
        scopes,
        issue_pattern,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_config ──────────────────────────────────────────────────────────

    #[test]
    fn test_parse_empty_toml() {
        let cfg = parse_config("").expect("empty TOML should parse fine");
        assert!(cfg.allow_custom_types.is_none());
        assert!(cfg.max_length.is_none());
        assert!(cfg.min_length.is_none());
        assert!(cfg.enforce_lowercase.is_none());
        assert!(cfg.disallow_period.is_none());
        assert!(cfg.enforce_lowercase_scope.is_none());
        assert!(cfg.require_scope.is_none());
        assert!(cfg.scopes.is_none());
        assert!(cfg.issue_pattern.is_none());
    }

    #[test]
    fn test_parse_all_fields() {
        let toml = r#"
allow_custom_types = false
max_length = 100
min_length = 10
enforce_lowercase = false
disallow_period = false
enforce_lowercase_scope = true
require_scope = true
scopes = "api,client"
issue_pattern = "^Refs: #\\d+"
"#;
        let cfg = parse_config(toml).expect("should parse");
        assert_eq!(cfg.allow_custom_types, Some(false));
        assert_eq!(cfg.max_length, Some(100));
        assert_eq!(cfg.min_length, Some(10));
        assert_eq!(cfg.enforce_lowercase, Some(false));
        assert_eq!(cfg.disallow_period, Some(false));
        assert_eq!(cfg.enforce_lowercase_scope, Some(true));
        assert_eq!(cfg.require_scope, Some(true));
        assert_eq!(cfg.scopes, Some("api,client".to_string()));
        assert_eq!(cfg.issue_pattern, Some(r"^Refs: #\d+".to_string()));
    }

    #[test]
    fn test_parse_partial_fields() {
        let toml = r#"
max_length = 50
require_scope = true
"#;
        let cfg = parse_config(toml).expect("should parse");
        assert!(cfg.allow_custom_types.is_none());
        assert_eq!(cfg.max_length, Some(50));
        assert!(cfg.min_length.is_none());
        assert!(cfg.enforce_lowercase.is_none());
        assert!(cfg.disallow_period.is_none());
        assert!(cfg.enforce_lowercase_scope.is_none());
        assert_eq!(cfg.require_scope, Some(true));
        assert!(cfg.scopes.is_none());
        assert!(cfg.issue_pattern.is_none());
    }

    #[test]
    fn test_parse_invalid_toml_returns_error() {
        let result = parse_config("this is not valid toml ][[[");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_wrong_type_returns_error() {
        let toml = r#"max_length = "not-a-number""#;
        let result = parse_config(toml);
        assert!(result.is_err());
    }

    // ── find_config_file ──────────────────────────────────────────────────────

    #[test]
    fn test_find_config_file_in_cwd() {
        let dir = tempdir();
        std::fs::write(dir.join(CONFIG_FILE_NAME), "").unwrap();
        let found = find_config_file(&dir);
        assert_eq!(found, Some(dir.join(CONFIG_FILE_NAME)));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_find_config_file_in_parent() {
        let dir = tempdir();
        let child = dir.join("subdir");
        std::fs::create_dir_all(&child).unwrap();
        std::fs::write(dir.join(CONFIG_FILE_NAME), "").unwrap();

        let found = find_config_file(&child);
        assert_eq!(found, Some(dir.join(CONFIG_FILE_NAME)));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_find_config_file_not_found() {
        // Use a deeply nested temp dir where no .commit-check.toml exists at any level
        // (we can't guarantee the filesystem root has none, so test via a known absence)
        let dir = tempdir();
        let found = find_config_file(&dir);
        // The temp dir itself has no config file; the result depends on ancestors.
        // We simply assert it doesn't find one inside our fresh dir.
        if let Some(ref path) = found {
            assert!(!path.starts_with(&dir));
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    // ── load_config_from_path ─────────────────────────────────────────────────

    #[test]
    fn test_load_config_from_path_ok() {
        let dir = tempdir();
        let path = dir.join(CONFIG_FILE_NAME);
        std::fs::write(&path, "max_length = 80\nrequire_scope = true\n").unwrap();

        let cfg = load_config_from_path(&path).expect("should load");
        assert_eq!(cfg.max_length, Some(80));
        assert_eq!(cfg.require_scope, Some(true));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_config_from_path_missing_file() {
        let path = PathBuf::from("/nonexistent/path/.commit-check.toml");
        let err = load_config_from_path(&path).expect_err("should fail");
        assert!(matches!(err, TomlError::Io { .. }));
    }

    #[test]
    fn test_load_config_from_path_invalid_content() {
        let dir = tempdir();
        let path = dir.join(CONFIG_FILE_NAME);
        std::fs::write(&path, "max_length = \"oops\"").unwrap();

        let err = load_config_from_path(&path).expect_err("should fail");
        assert!(matches!(err, TomlError::Parse { .. }));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn tempdir() -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("cc_config_test_{}_{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
