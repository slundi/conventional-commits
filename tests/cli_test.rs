use assert_cmd::Command;
use predicates::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn get_test_command() -> Command {
    Command::cargo_bin("commit-check").expect("Failed to find commit-check binary")
}

fn make_tmp_git_dir() -> std::path::PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("cc_cli_test_{}_{}", std::process::id(), id));
    std::fs::create_dir_all(dir.join(".git/hooks")).unwrap();
    dir
}

#[test]
fn test_valid_commit_via_arg() {
    let mut cmd = get_test_command();
    cmd.arg("--message")
        .arg("feat: add new feature")
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid conventional commit"));
}

#[test]
fn test_invalid_commit_via_arg() {
    let mut cmd = get_test_command();
    cmd.arg("--message")
        .arg("invalid commit")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid commit message"));
}

#[test]
fn test_examples_command() {
    let mut cmd = get_test_command();
    cmd.arg("examples")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Examples of valid Conventional Commits",
        ));
}

#[test]
fn test_types_command() {
    let mut cmd = get_test_command();
    cmd.arg("types")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Standard Conventional Commit types",
        ));
}

#[test]
fn test_verbose_output() {
    let mut cmd = get_test_command();
    cmd.arg("--message")
        .arg("feat(auth): add OAuth support")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Type: feat"))
        .stdout(predicate::str::contains("Scope: auth"));
}

#[test]
fn test_breaking_change_detection() {
    let mut cmd = get_test_command();
    cmd.env("NO_COLOR", "1");
    cmd.arg("--message")
        .arg("feat!: breaking API change")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("BREAKING CHANGE detected"));
}

#[test]
fn test_custom_config() {
    let mut cmd = get_test_command();
    cmd.arg("--message")
        .arg("feat: add feature")
        .arg("--max-length")
        .arg("10")
        .assert()
        .failure()
        .stderr(predicate::str::contains("too long"));
}

#[test]
fn test_stdin_input() {
    let mut cmd = get_test_command();
    cmd.write_stdin("feat: add new feature\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid conventional commit"));
}

#[test]
fn test_fix_missing_space() {
    get_test_command()
        .arg("--message")
        .arg("feat:Add thing.")
        .arg("--fix")
        .assert()
        .success()
        .stdout(predicate::str::contains("feat: add thing"));
}

#[test]
fn test_fix_uppercase_and_period() {
    get_test_command()
        .arg("--message")
        .arg("fix: Resolve bug.")
        .arg("--fix")
        .assert()
        .success()
        .stdout(predicate::str::contains("fix: resolve bug"));
}

#[test]
fn test_fix_already_valid_no_fixed_line() {
    get_test_command()
        .arg("--message")
        .arg("feat: add thing")
        .arg("--fix")
        .assert()
        .success()
        .stdout(predicate::str::contains("Fixed:").not());
}

#[test]
fn test_fix_invalid_even_after_fix() {
    // A completely unparsable message can't be fixed — should still fail
    get_test_command()
        .arg("--message")
        .arg("not a conventional commit")
        .arg("--fix")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid commit message"));
}

#[test]
fn test_install_hook_command() {
    let tmp = make_tmp_git_dir();

    get_test_command()
        .current_dir(&tmp)
        .arg("install-hook")
        .assert()
        .success()
        .stdout(predicate::str::contains("Git hook installed successfully"));

    assert!(tmp.join(".git/hooks/commit-msg").exists());
    std::fs::remove_dir_all(&tmp).unwrap();
}

#[test]
fn test_install_hook_already_exists_fails() {
    let tmp = make_tmp_git_dir();
    std::fs::write(tmp.join(".git/hooks/commit-msg"), "existing hook").unwrap();

    get_test_command()
        .current_dir(&tmp)
        .arg("install-hook")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    // Existing hook must not be overwritten
    let content = std::fs::read_to_string(tmp.join(".git/hooks/commit-msg")).unwrap();
    assert_eq!(content, "existing hook");

    std::fs::remove_dir_all(&tmp).unwrap();
}

#[test]
fn test_scopes_allowed() {
    get_test_command()
        .arg("--message")
        .arg("feat(api): add endpoint")
        .arg("--scopes")
        .arg("api,client,shared")
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid conventional commit"));
}

#[test]
fn test_cli_allowed_types_success() {
    let mut cmd = get_test_command();
    cmd.arg("--message")
        .arg("api: add new endpoint")
        .arg("--types")
        .arg("feat,fix,api")
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid conventional commit"));
}

#[test]
fn test_scopes_rejected() {
    get_test_command()
        .arg("--message")
        .arg("feat(ui): add button")
        .arg("--scopes")
        .arg("api,client,shared")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not in the allowed scopes"));
}

#[test]
fn test_no_scope_passes_with_scopes_flag() {
    get_test_command()
        .arg("--message")
        .arg("feat: add thing")
        .arg("--scopes")
        .arg("api,client")
        .assert()
        .success();
}

#[test]
fn test_issue_pattern_valid_github_ref() {
    get_test_command()
        .arg("--message")
        .arg("feat: add feature\n\nRefs: #123")
        .arg("--issue-pattern")
        .arg(r"^Refs: #\d+")
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid conventional commit"));
}

#[test]
fn test_issue_pattern_valid_jira_ref() {
    get_test_command()
        .arg("--message")
        .arg("fix: resolve crash\n\nFixes: PROJ-456")
        .arg("--issue-pattern")
        .arg(r"^Fixes: [A-Z]+-\d+")
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid conventional commit"));
}

#[test]
fn test_issue_pattern_missing_footer_fails() {
    get_test_command()
        .arg("--message")
        .arg("feat: add feature")
        .arg("--issue-pattern")
        .arg(r"^Refs: #\d+")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No footer matches the required issue pattern",
        ));
}

#[test]
fn test_issue_pattern_non_matching_footer_fails() {
    get_test_command()
        .arg("--message")
        .arg("feat: add feature\n\nReviewed-by: Alice")
        .arg("--issue-pattern")
        .arg(r"^Refs: #\d+")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No footer matches the required issue pattern",
        ));
}

#[test]
fn test_issue_pattern_not_set_does_not_require_footer() {
    get_test_command()
        .arg("--message")
        .arg("feat: add feature")
        .assert()
        .success();
}

#[test]
fn test_install_hook_force_overwrites_existing() {
    let tmp = make_tmp_git_dir();
    std::fs::write(tmp.join(".git/hooks/commit-msg"), "existing hook").unwrap();

    get_test_command()
        .current_dir(&tmp)
        .arg("install-hook")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Git hook installed successfully"));

    let content = std::fs::read_to_string(tmp.join(".git/hooks/commit-msg")).unwrap();
    assert!(content.contains("commit-check"));

    std::fs::remove_dir_all(&tmp).unwrap();
}

#[test]
fn test_cli_allowed_types_failure() {
    let mut cmd = get_test_command();
    cmd.arg("--message")
        .arg("docs: update readme")
        .arg("--types")
        .arg("feat,fix")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Type 'docs' is not allowed"))
        .stderr(predicate::str::contains("Allowed types are: feat, fix"));
}

#[test]
fn test_cli_allowed_types_with_verbose() {
    let mut cmd = get_test_command();
    cmd.arg("--message")
        .arg("feat: working")
        .arg("--types")
        .arg("feat")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Type: feat"));
}

#[test]
fn test_json_output_escapes_control_characters() {
    let mut cmd = get_test_command();
    let output = cmd
        .arg("--format")
        .arg("json")
        .arg("--message")
        .arg("feat: add \u{1b}[1mbold\u{1b}[0m and\u{08}backspace")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    let json_line = stdout.trim_end_matches('\n');

    assert!(
        json_line.contains(r#""description":"add \u001b[1mbold\u001b[0m and\bbackspace""#),
        "{json_line}"
    );
    assert!(
        json_line.chars().all(|character| character >= '\u{20}'),
        "raw control character in JSON output: {json_line:?}"
    );
}
