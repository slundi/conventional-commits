# Integration

## Github Actions

You can run the check on push or pull_request events.

```yaml
name: Commit Check
on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Validate Commit Message
        run: |
          # Get the latest commit message and pipe to the tool
          git log -1 --pretty=%B | docker run -i slundi/commit-check:latest validate
```
**Note**: If using the binary directly, ensure it is in your PATH or use the absolute path.

## GitLab CI/CD

GitLab provides the commit message in the `CI_COMMIT_MESSAGE` variable.

```yaml
commit-msg-check:
  image: slundi/commit-check:latest
  stage: test
  script:
    - echo "$CI_COMMIT_MESSAGE" | commit-check validate
  only:
    - merge_requests
    - pushes
```

## Woodpecker CI / Drone CI

Both Woodpecker and Drone use a similar syntax. You can use the CI_COMMIT_MESSAGE environment variable.

```yaml
steps:
  validate-commit:
    image: slundi/commit-check:latest
    commands:
      - echo "$${CI_COMMIT_MESSAGE}" | commit-check validate --verbose
```
Using `--verbose` helps debug failures by showing the parsed components like Type, Scope, and Description.

## Local Development (Git Hook)

To catch invalid messages before they even reach the pipeline, install the git hook locally:

```bash
commit-check install-hook
```

This creates a `.git/hooks/commit-msg` script that automatically validates your message every time you run `git commit`.

## Pre commit

### 1. Create `.pre-commit-hooks.yaml`

Place this file in the root of your project. It tells the framework how to run your Rust binary.
```yaml
- id: commit-check
  name: Conventional Commit Checker
  description: Check commit messages for Conventional Commits compliance.
  entry: commit-check validate --message
  language: rust
  stages: [commit-msg]
  always_run: true
```

### 2. Usage in a Project

Once that file is in your repo, any project can use your tool by adding this to their `.pre-commit-config.yaml`:
```yaml
repos:
  - repo: https://codeberg.org/slundi/conventional-commits
    rev: v0.9.4  # Use a specific tag or commit hash
    hooks:
      - id: commit-check
        stages: [commit-msg]
```

## Nix

### `shell.nix` (Traditional Nix)

This is useful for providing a development environment where the binary is automatically built and available in the `$PATH`.

```nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    gcc
    # Standard dependencies for your tool
    libiconv
  ];

  shellHook = ''
    echo "🔨 Conventional Commits Dev Shell"
    # Optional: Alias to run the local version easily
    alias check="cargo run -- validate"
  '';
}
```

### `flake.nix` (Modern Nix)

Run it directly with `nix run github:slundi/conventional-commits -- validate --message "feat: nix support`.

## Comparison of Installation Methods

<table>
<tr><th>Feature</th><td>`commit-check install-hook`</td><td>`pre-commit` Framework</td></tr>
<tr><th>Dependencies</th><td>None (Standalone binary)</td><td>Requires Python & `pre-commit`</td></tr>
<tr><th>Setup</th><td>Manual command execution</td><td>Managed via `.pre-commit-config.yaml`</td></tr>
<tr><th>Binary Source</th><td>Uses your local compiled version</td><td>Automatically downloads/builds from source</td></tr>
<tr><th>Execution</th><td>Runs `commit-msg` hook directly</td><td>Wraps execution in its own environment</td></tr>
<table>
