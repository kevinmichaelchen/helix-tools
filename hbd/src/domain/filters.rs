//! Filtering logic for issue queries.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::types::{Issue, Status};

pub fn ready_issues<'a>(
    issues: impl IntoIterator<Item = &'a Issue>,
    all_issues: &HashMap<String, Issue>,
) -> Vec<&'a Issue> {
    issues
        .into_iter()
        .filter(|i| i.status == Status::Open)
        .filter(|i| {
            i.depends_on.iter().all(|dep| {
                all_issues
                    .get(&dep.id)
                    .is_none_or(|blocker| blocker.status == Status::Closed)
            })
        })
        .collect()
}

pub fn blocked_issues<'a>(
    issues: impl IntoIterator<Item = &'a Issue>,
    all_issues: &HashMap<String, Issue>,
) -> Vec<&'a Issue> {
    issues
        .into_iter()
        .filter(|i| i.status != Status::Closed)
        .filter(|i| {
            i.depends_on.iter().any(|dep| {
                all_issues
                    .get(&dep.id)
                    .is_some_and(|blocker| blocker.status != Status::Closed)
            })
        })
        .collect()
}

pub fn stale_issues(issues: Vec<Issue>, cutoff: DateTime<Utc>) -> Vec<Issue> {
    issues
        .into_iter()
        .filter(|i| i.updated_at < cutoff && i.status != Status::Closed)
        .collect()
}
