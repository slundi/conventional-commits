# Roadmap

## 1. Gitea/GitHub/Jira Integration (Issue Refs)

The Conventional Commits spec often includes a footer for issue tracking (e.g., `Refs: #123` or `Fixes: PROJ-456`).
* **Feature**: Add an `--issue-pattern` regex arg.
* **Action**: The tool validates that at least one footer matches project's issue format.
* **Why it shines**: It ensures traceability between commits and ticketing system.
* **Complexity**: Low. We already use `regex` in the project. This is just one additional regex check on the footer lines.
* **Weight**: Zero. We are already paying the "cost" of the `regex` crate.
* **Verdict**: Great "pro" feature. It’s essentially "free" from a performance standpoint.

## 2. Commit Message Template Generator

Sometimes developers forget the `types` even with custom types command.
* **Feature**: A command like `commit-check init-msg`.
* **Action**: It could output a commented-out template that opens in their editor, similar to how git's default `COMMIT_EDITMSG` works, but with allowed types listed in the comments.
* **Complexity**: Low. Just a few `println!` statements or writing a static string to a temporary file.
* **Weight**: Zero.
* **Verdict**: Nice-to-have, but perhaps less "lightweight" in terms of UX. Most developers prefer their editor to handle templates.

## 3. Summary Statistics (`--stats`)

If we are running this in a CI pipeline over a range of commits (e.g., git log main..feature), it’s nice to see a summary.
* **Feature**: `commit-check check-range main..HEAD`.
* **Action**: It prints a summary: "Validated 12 commits: 10 Pass, 2 Fail (fix: ... , feat: ...)."
* **Complexity**: Medium. We would need to change the CLI to accept a range of commits, which involves calling `git log` internally or having a git crate in the project.
* **Weight**: Medium. We might be tempted to pull in a crate like `git2` (which is very heavy) or use `std::process::Command` to call the system `git` (lightweight but requires git to be installed).
* **Verdict**: Use `std::process::Command` to keep it light, but avoid `git2` at all costs we you want to stay under 1MB.

## 4. Configuration File (.commit-check.toml)

While CLI args are great for CI, having a persistent config file makes local development easier.
* Feature: Automatically look for a `.commit-check.toml` in the repo root.
* Why it shines: You don't have to pass `--types` or `--max-length` every time you run the command manually; it just "knows" the project rules.
* Complexity: Medium. You need logic to search parent directories for the file.
* **Weight**: High. To parse `.toml`, we would typically add the `toml` and `serde` crates.
  * Standard path: Adds ~150KB to the binary and increases compile time.
  * Lightweight path: Use a tiny parser like `micro-toml` or simply use a JSON config (since `serde_json` is very optimized) or even a simple `.env` style key-value parser.
* **Verdict**: The biggest risk to "lightweight" status. If we implement this, choose the parsing library very carefully.
