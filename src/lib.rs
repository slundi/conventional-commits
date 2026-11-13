//! # Conventional Commits Validator
//!
//! A lightweight library for validating Git commit messages according to the
//! [Conventional Commits](https://conventionalcommits.org/) specification.
//!
//! ## Usage
//!
//! ```rust
//! use conventional_commits::{validate_commit, CommitType};
//!
//! let message = "feat: add new user authentication";
//! match validate_commit(message) {
//!     Ok(commit) => {
//!         println!("Valid commit: {:?}", commit.commit_type);
//!     }
//!     Err(e) => {
//!         eprintln!("Invalid commit: {}", e);
//!     }
//! }
//! ```

mod clean;
mod commit;
mod error;
mod fix;
mod parse;
mod validate;

pub use clean::{CleanResult, clean_commit_body};
pub use commit::{CommitType, ConventionalCommit};
pub use error::{CommitValidationError, ValidationErrors};
pub use fix::fix_commit_message;
pub use validate::{ValidationConfig, validate_commit, validate_commit_with_config};
