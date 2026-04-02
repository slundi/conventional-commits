# Roadmap

## 1. Commit Message Template Generator

Sometimes developers forget the `types` even with custom types command.
* **Feature**: A command like `commit-check init-msg`.
* **Action**: It could output a commented-out template that opens in their editor, similar to how git's default `COMMIT_EDITMSG` works, but with allowed types listed in the comments.
* **Complexity**: Low. Just a few `println!` statements or writing a static string to a temporary file.
* **Weight**: Zero.
* **Verdict**: Nice-to-have, but perhaps less "lightweight" in terms of UX. Most developers prefer their editor to handle templates.

## 2. Summary Statistics (`--stats`)

If we are running this in a CI pipeline over a range of commits (e.g., git log main..feature), it’s nice to see a summary.
* **Feature**: `commit-check check-range main..HEAD`.
* **Action**: It prints a summary: "Validated 12 commits: 10 Pass, 2 Fail (fix: ... , feat: ...)."
* **Complexity**: Medium. We would need to change the CLI to accept a range of commits, which involves calling `git log` internally or having a git crate in the project.
* **Weight**: Medium. We might be tempted to pull in a crate like `git2` (which is very heavy) or use `std::process::Command` to call the system `git` (lightweight but requires git to be installed).
* **Verdict**: Use `std::process::Command` to keep it light, but avoid `git2` at all costs we you want to stay under 1MB.

## 3. Custom clean rules

When you use automated tools, you may have `Co-authored ...`, `Signed-off-by ...` or similar strings in the end of the body message
* **Feature**: add 1 or multiple clean line rules in the CLI and the TOML.
* **Action**: remove unwanted lines from the commit body and ensure there is no more than 1 line between text bodies, also trim sends spaces at the end of the body. We must show in the output removed lines
* **Complexity**: Medium, we can have simple filters using start with and other filters using regex
* **Weight**: Low. logic and we reuse already available crates.
* **Verdict**: Nice-to-have since even AI are adding `Co-authored` stuffs even when told to not do becouse they "forgot"
