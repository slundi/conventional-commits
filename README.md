# Conventional Commits Validator

A lightweight Rust library and CLI tool for validating Git commit messages according to the [Conventional Commits](https://conventionalcommits.org/) specification.

## Features

- ✅ Validates commit messages against Conventional Commits v1.0.0
- 🔧 Configurable validation rules
- 📚 Can be used as a library or CLI tool
- 🚀 Lightweight with minimal dependencies
- 🎯 Supports all standard commit types
- 🔍 Detects breaking changes
- 📖 Provides helpful error messages and examples

## Installation

### From crates.io

```bash
cargo install conventional-commits
```

### From source

```bash
git clone https://codeberg.org/slundi/conventional-commits
cd conventional-commits
cargo build --release
```

## CLI Usage

### Basic validation

```bash
# Validate a commit message
commit-check --message "feat: add user authentication"

# Read from stdin
echo "fix: resolve login issue" | commit-check

# Validate with verbose output
commit-check --message "feat(auth): add OAuth support" --verbose
```

### Commands

```bash
# Show examples of valid commits
commit-check examples

# Show available commit types
commit-check types

# Validate with custom configuration
commit-check --message "feat: new feature" --max-length 50 --allow-custom-types=false
```

### Configuration Options

- `--max-length <N>`: Maximum description length (default: 72)
- `--allow-custom-types <BOOL>`: Allow custom commit types (default: true)
- `--enforce-lowercase <BOOL>`: Enforce lowercase description (default: true)
- `--disallow-period <BOOL>`: Disallow period at end of description (default: true)
- `--verbose`: Enable detailed output

### Integration

See [Integration](Integration.md).

## Conventional Commits Format

The Conventional Commits specification defines the following format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Standard Types

- `feat` - A new feature
- `fix` - A bug fix
- `docs` - Documentation only changes
- `style` - Changes that do not affect the meaning of the code
- `refactor` - A code change that neither fixes a bug nor adds a feature
- `perf` - A code change that improves performance
- `test` - Adding missing tests or correcting existing tests
- `build` - Changes to the build process or auxiliary tools
- `ci` - Changes to CI configuration files and scripts
- `chore` - Other changes that don't modify src or test files
- `revert` - Reverts a previous commit

### Examples

```
feat: add email notifications
fix(auth): resolve token expiration issue
feat!: remove deprecated API endpoints
docs(readme): update installation instructions

feat: add user profiles

Allow users to create and customize their profiles
with avatar upload and bio sections.

Closes #123
BREAKING CHANGE: User model schema has changed
```

## Git Hooks Integration

You can integrate this tool with Git hooks to automatically validate commit messages:

### commit-msg hook

Create `.git/hooks/commit-msg`:

```bash
#!/bin/sh
commit-check --message "$(cat "$1")"
```

Make it executable:

```bash
chmod +x .git/hooks/commit-msg
```

### Pre-commit integration

Add to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: conventional-commits
        name: Conventional Commits
        entry: commit-check
        language: system
        stages: [commit-msg]
        args: ["--message"]
```

## API Reference

### Core Functions

- `validate_commit(message: &str)` - Validates a commit message with default configuration
- `validate_commit_with_config(message: &str, config: &ValidationConfig)` - Validates with custom configuration

### Types

- `ConventionalCommit` - Represents a parsed commit
- `CommitType` - Enum of commit types
- `ValidationConfig` - Configuration for validation rules
- `CommitValidationError` - Error types for validation failures

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development

See [Development](Development.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Conventional Commits](https://conventionalcommits.org/) specification
- [commitlint](https://github.com/conventional-changelog/commitlint) for inspiration
- [commitizen](https://github.com/commitizen/cz-cli) for reference implementation
