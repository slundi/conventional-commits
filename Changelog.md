# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] - 2025-08-22

### Added
- Nix package support with flakes
- Shell completions preparation
- Comprehensive test suite
- Initial release of conventional-commits
- Library (`conventional_commits`) for validating Conventional Commits
- CLI tool (`commit-check`) for command-line validation
- Support for all standard Conventional Commit types:
  - `feat` - New features
  - `fix` - Bug fixes  
  - `docs` - Documentation changes
  - `style` - Code style changes
  - `refactor` - Code refactoring
  - `perf` - Performance improvements
  - `test` - Test changes
  - `build` - Build system changes
  - `ci` - CI configuration changes
  - `chore` - Other changes
  - `revert` - Revert commits
- Custom commit type support
- Breaking change detection (both `!` syntax and footer syntax)
- Configurable validation rules:
  - Maximum description length
  - Lowercase description enforcement
  - Period disallowing at end of description
  - Custom type allowance
- Scope parsing and validation
- Multi-line commit message support with body and footers
- Comprehensive error messages with helpful suggestions
- Examples and help commands
- Git hook integration scripts
- Stdin input support
- Verbose output mode

### Technical Details
- Written in Rust 2021 edition
- Minimal dependencies (clap, regex, thiserror)
- Comprehensive test suite with unit and integration tests
- Proper error handling with custom error types
- Memory-safe and performant implementation
- Cross-platform support (Linux, macOS, Windows)

### Documentation
- Complete README with usage examples
- API documentation for library usage
- Git hook setup instructions
- CI/CD integration examples
- Makefile with common development tasks

---

## Release Process

### For Maintainers

To create a new release:

1. Update version in `Cargo.toml`
2. Update version in `default.nix` (for Nix package)
3. Update this CHANGELOG.md with new changes
4. Commit changes: `git commit -m "chore: prepare release v0.X.Y"`
5. Create and push tag: `git tag v0.X.Y && git push origin v0.X.Y`
6. Publish to crates.io: `cargo publish`
7. Update nixpkgs (if applicable)

### Version Numbering

- **Major (X.0.0)**: Breaking changes to public API
- **Minor (0.X.0)**: New features, backwards compatible
- **Patch (0.0.X)**: Bug fixes, backwards compatible

### Breaking Changes

Any changes that modify the public API of the library or change the CLI interface in non-backwards-compatible ways constitute breaking changes and require a major version bump.