set show-recipe-help := true

# --- Variables ---
binary_name := "rust_template" # Change this to your project name

# --- Default ---
[help]
default:
    @just --list

# --- Development ---

# Build the project in debug mode
build:
    cargo build

# Run the project
run *args:
    cargo run -- {{args}}

# Watch for changes and run (requires cargo-watch)
watch:
    cargo watch -x run

# --- Quality Control ---

# Run all tests
test:
    cargo nextest run

# Run a full health check (Vulnerabilities, Unmaintained crates, Licenses, Unused Deps)
health-check:
    cargo deny check
    cargo machete

# Report direct dependencies with a newer release available (informational, never fails)
outdated:
    cargo outdated --root-deps-only --exit-code 0

# Run clippy with strict warnings
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Run all pre-commit hooks manually on all files
check:
    prek run --all-files

# Force update hooks
update-hooks:
    prek autoupdate

# Run gitleaks to scan for secrets
scan-secrets:
    gitleaks detect --verbose --redact

# Generate lcov.info for editor coverage display (vscode-coverage-gutters)
coverage:
    cargo llvm-cov --lcov --output-path lcov.info

# Check that overall line coverage meets the 80% threshold
# BASE is accepted for interface compatibility with CI (e.g.: just coverage-check master)
coverage-check BASE="master":
    cargo build -p dgaard-monitor
    cargo llvm-cov --lcov --output-path lcov.info
    cargo llvm-cov report --fail-under-lines 80

# --- Cleanup ---

# Clean build artifacts
clean:
    cargo clean

# --- CI Simulation ---

# Run the full pipeline as it would run in CI
ci: fmt lint test
    @echo "✅ All checks passed!"

# --- Release ---

# Create and push an annotated release tag, triggering the release workflow
# Usage: just tag v1.2.3
tag VERSION:
    git tag -a {{VERSION}} -m "Release {{VERSION}}"
    git push origin {{VERSION}}
    git push Github {{VERSION}}
    @echo "Pushed tag {{VERSION}} — release workflow will start on Codeberg"

# Build a release binary for the current platform
build-release:
    cargo build --release

# Build a release binary for a specific cross-compilation target
# Requires `cross` in your PATH (included in flake.nix devShell)
# Usage: just build-release-cross aarch64-unknown-linux-gnu
build-release-cross TARGET:
    cross build --release --target {{TARGET}}

# Cross-compile dgaard for aarch64-musl (OpenWrt on MediaTek/Filogic routers
# such as GL-MT6000 Flint 2, Banana Pi R3). Output is at ./result/bin/dgaard.
nix-release-build:
    nix build .#dgaard-aarch64-musl
    @echo "→ dgaard (aarch64-musl): $(readlink -f result)/bin/dgaard"

# Cross-compile dgaard-monitor for aarch64-musl. Output at ./result/bin/dgaard-monitor.
nix-release-build-monitor:
    nix build .#dgaard-monitor-aarch64-musl -o result-monitor
    @echo "→ dgaard-monitor (aarch64-musl): $(readlink -f result-monitor)/bin/dgaard-monitor"

# Build both dgaard and dgaard-monitor for aarch64-musl
nix-release-build-all: nix-release-build nix-release-build-monitor
