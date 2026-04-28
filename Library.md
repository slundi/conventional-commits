# Library usage

Add to your `Cargo.toml`:

```toml
[dependencies]
conventional-commits = "0.1.0"
```

## Basic validation

```rust
use conventional_commits::{validate_commit, CommitType};

fn main() {
    let message = "feat: add user authentication";

    match validate_commit(message) {
        Ok(commit) => {
            println!("Valid commit!");
            println!("Type: {:?}", commit.commit_type);
            println!("Description: {}", commit.description);

            if commit.is_breaking_change() {
                println!("⚠️ Breaking change detected!");
            }
        }
        Err(e) => {
            eprintln!("Invalid commit: {}", e);
        }
    }
}
```

## Custom validation

```rust
use conventional_commits::{validate_commit_with_config, ValidationConfig};

let config = ValidationConfig {
    max_description_length: 50,
    enforce_lowercase_description: true,
    disallow_description_period: true,
    ..ValidationConfig::default()
};

let result = validate_commit_with_config("feat: add feature", &config);
```

## Body cleaning

`clean_commit_body` removes unwanted lines from the commit body before validation.
It also collapses consecutive blank lines and trims trailing whitespace.

```rust
use conventional_commits::clean_commit_body;

let raw = "feat: add thing\n\nBody text.\nCo-authored-by: Alice <a@x.com>\nSigned-off-by: Bob";

// starts_with rules — fast prefix match, no regex overhead
let result = clean_commit_body(raw, &["Co-authored-by", "Signed-off-by"], &[]).unwrap();
println!("{}", result.cleaned_message);
// feat: add thing
//
// Body text.

for line in &result.removed_lines {
    println!("Stripped: {line}");
}

// regex rules — for more complex patterns
let result = clean_commit_body(raw, &[], &[r"^🤖.*", r"^Generated-by:"]).unwrap();

// Both rule types can be combined in a single call
let result = clean_commit_body(
    raw,
    &["Co-authored-by"],
    &[r"^🤖.*"],
).unwrap();
```
