//! hbd - Git-first issue tracker library
//!
//! This crate provides the core functionality for the hbd issue tracker.
//! Issues are stored as Markdown files with YAML frontmatter in `.tickets/`.
//!
//! # Quick Start
//!
//! ```no_run
//! use hbd::{TicketStore, Issue, Status};
//!
//! // Find and connect to the project store
//! let store = TicketStore::from_current_dir()?;
//!
//! // List all issues
//! let issues = store.read_all_issues()?;
//!
//! // Get a specific issue
//! let issue = store.read_issue("bd-abc123")?;
//!
//! // Update an issue's status
//! let mut issue = store.read_issue("bd-abc123")?;
//! issue.status = Status::InProgress;
//! store.write_issue(&issue)?;
//! # Ok::<(), hbd::HbdError>(())
//! ```

pub mod db;
pub mod domain;
pub mod error;
pub mod markdown;
pub mod storage;
pub mod types;

mod id;

pub use error::{HbdError, Result};
pub use id::generate_issue_id;
pub use storage::TicketStore;
pub use types::{
    Comment, CreatorType, DepType, Dependency, Issue, IssueType, Label, Priority, Status,
};
