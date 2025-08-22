use conventional_commits::{
    validate_commit, validate_commit_with_config, CommitType, ValidationConfig,
};

fn main() {
    println!("=== Basic Validation Examples ===\n");

    // Example 1: Valid commits
    let valid_commits = vec![
        "feat: add user authentication",
        "fix: resolve memory leak in parser",
        "feat(auth): implement OAuth2 flow",
        "feat!: breaking change to API",
        "docs: update README with new examples",
    ];

    for commit in valid_commits {
        match validate_commit(commit) {
            Ok(parsed) => {
                println!("✅ Valid: {}", commit);
                println!("   Type: {}", parsed.commit_type);
                if let Some(scope) = &parsed.scope {
                    println!("   Scope: {}", scope);
                }
                println!("   Description: {}", parsed.description);
                if parsed.is_breaking_change() {
                    println!("   ⚠️ Breaking change detected!");
                }
                println!();
            }
            Err(e) => {
                println!("❌ Error validating '{}': {}", commit, e);
            }
        }
    }

    println!("=== Invalid Commits Examples ===\n");

    // Example 2: Invalid commits
    let invalid_commits = vec![
        "add user authentication",        // Missing type
        "feat:",                          // Missing description
        "feat: Add user authentication",  // Uppercase description
        "feat: add user authentication.", // Period at end
        "",                               // Empty message
    ];

    for commit in invalid_commits {
        match validate_commit(commit) {
            Ok(_) => {
                println!("✅ Unexpectedly valid: {}", commit);
            }
            Err(e) => {
                println!("❌ Invalid: {} - {}", commit, e);
            }
        }
    }

    println!("\n=== Custom Configuration Example ===\n");

    // Example 3: Custom configuration
    let config = ValidationConfig {
        max_description_length: 30,           // Shorter limit
        allow_custom_types: false,            // Only standard types
        enforce_lowercase_description: false, // Allow uppercase
        disallow_description_period: false,   // Allow periods
    };

    let test_commits = vec![
        "feat: Add a very long description that exceeds the limit",
        "custom: custom type commit",
        "feat: Add Feature.",
        "feat: Short desc",
    ];

    for commit in test_commits {
        match validate_commit_with_config(commit, &config) {
            Ok(parsed) => {
                println!("✅ Valid with custom config: {}", commit);
                println!("   Type: {}", parsed.commit_type);
            }
            Err(e) => {
                println!("❌ Invalid with custom config: {} - {}", commit, e);
            }
        }
    }

    println!("\n=== Breaking Change Detection ===\n");

    // Example 4: Breaking changes
    let breaking_commits = vec![
        "feat!: remove deprecated API",
        "feat: add new feature\n\nBREAKING CHANGE: API response format changed",
        "refactor: restructure codebase\n\nThis change breaks backwards compatibility\n\nBREAKING-CHANGE: Configuration format updated",
    ];

    for commit in breaking_commits {
        match validate_commit(commit) {
            Ok(parsed) => {
                println!("Commit: {}", commit.lines().next().unwrap_or(""));
                println!("Breaking change: {}", parsed.is_breaking_change());
                if !parsed.footers.is_empty() {
                    println!("Footers: {:?}", parsed.footers);
                }
                println!();
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
    }

    println!("=== Commit Type Examples ===\n");

    // Example 5: Different commit types
    let type_examples = vec![
        ("feat", "A new feature"),
        ("fix", "A bug fix"),
        ("docs", "Documentation changes"),
        ("style", "Code style changes"),
        ("refactor", "Code refactoring"),
        ("perf", "Performance improvements"),
        ("test", "Adding tests"),
        ("build", "Build system changes"),
        ("ci", "CI configuration changes"),
        ("chore", "Other changes"),
    ];

    for (type_str, description) in type_examples {
        let commit_type = CommitType::from(type_str);
        println!(
            "{}: {} - Example: '{}: {}'",
            type_str,
            description,
            commit_type,
            match type_str {
                "feat" => "add user registration",
                "fix" => "resolve login timeout",
                "docs" => "update API documentation",
                "style" => "format code with rustfmt",
                "refactor" => "extract validation logic",
                "perf" => "optimize database queries",
                "test" => "add integration tests",
                "build" => "update cargo dependencies",
                "ci" => "add GitHub Actions workflow",
                "chore" => "update version numbers",
                _ => "example description",
            }
        );
    }
}
