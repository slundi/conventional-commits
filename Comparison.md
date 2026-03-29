# Comparison

The landscape of commit validation is dominated by JavaScript-based tools like commitlint and commitizen, often paired with Husky. This Rust-based `commit-check` stands out by offering a "native" feel that is significantly faster and easier to deploy, especially in environments where a Node.js runtime is not desired.

| Feature | commitlint | commitizen | This tool (`commit-check`) |
|:--------|:-----------|:-----------|:---------------------------|
| **Primary Goal** | Validation/Linting | Interactive Creation (Wizard) | **Validation + Hook Management** |
| **Runtime** | Node.js (Heavy) | Node.js / Python (Heavy) | **Rust (Native/Static Binary)** |
| **Performance** | Slower (VM startup time) | Moderate | **Instant (Compiled binary)** |
| **Setup Complexity** | High (Husky + config + NPM) | Moderate | **Low** (`install-hook` command) |
| **CI Integration** | Requires Node environment | Requires runtime | **Lightweight Docker/Single Binary** |
| **Breaking Changes** | Regex-based check | Guided prompt | **Semantic Footer Analysis** |

## Where it shines

### 1. Zero Dependency Deployment

The biggest pain point with commitlint is the dependency on Node.js. To use it, a developer must have npm installed, and the project often ends up with a node_modules folder just for git hooks.

* Advantage: this `commit-check` is a single binary. It can be downloaded and run on a machine that has nothing but Git installed. This is huge for Go, Rust, or C++ projects that want to avoid "polluting" their environment with the JS ecosystem.

### 2. Native Performance

While a few milliseconds might seem trivial, "hook lag" is a real psychological barrier for developers.

* **Advantage**: Because it’s written in Rust, the tool starts and finishes the check before a Node.js process would even finish initializing its virtual machine. This makes the `git commit` experience feel seamless.

### 3. Integrated Hook Management

Standard tools require Husky or pre-commit to actually link the linter to Git.

* **Advantage**: with `install-hook` command, it removes the "middleman." It handles the logic of writing to `.git/hooks/commit-msg` itself, providing a vertically integrated solution that is much easier to explain to a new team member.

### 4. Semantic Parsing vs. Simple Linting

Many linters just use a single regex for the header.

* **Advantage**: this tool actually parses the body and footers. By using a `Peekable` iterator to ensure `BREAKING CHANGE` only triggers if it's at the very end of the message, you are more compliant with the actual Conventional Commits 1.0.0 spec than many basic linter configurations.
