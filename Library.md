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
    allow_custom_types: false,
    enforce_lowercase_description: true,
    disallow_description_period: true,
};

let result = validate_commit_with_config("feat: add feature", &config);
```
