use assert_cmd::Command;
use predicates::prelude::*;

fn get_test_command() -> Command {
    Command::cargo_bin("commit-check").expect("Failed to find commit-check binary")
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
    cmd.arg("--message")
        .arg("feat!: breaking API change")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Breaking change detected"));
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
