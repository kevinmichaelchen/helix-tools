#![allow(dead_code)]

use std::fmt::Write;
use std::path::Path;

use chrono::{DateTime, Utc};
use gray_matter::{Matter, Pod, engine::YAML};
use serde::{Deserialize, Serialize};

use crate::error::{HbdError, Result};
use crate::types::{Comment, CreatorType, DepType, Dependency, Issue, IssueType, Priority, Status};

#[derive(Debug, Serialize, Deserialize)]
struct IxchelFrontMatter {
    id: String,
    title: String,
    #[serde(rename = "type")]
    entity_type: String,
    status: String,
    priority: u8,
    issue_type: String,
    created_at: String,
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    closed_at: Option<String>,
    created_by: String,
    #[serde(default = "default_creator_type")]
    created_by_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    depends_on: Vec<DependencyFrontMatter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    estimated_minutes: Option<i32>,
}

fn default_creator_type() -> String {
    "human".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct LegacyFrontMatter {
    id: String,
    title: String,
    status: String,
    priority: u8,
    #[serde(rename = "type")]
    issue_type: String,
    created_at: String,
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    closed_at: Option<String>,
    created_by: String,
    #[serde(default = "default_creator_type")]
    created_by_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    depends_on: Vec<DependencyFrontMatter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    estimated_minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AnyFrontMatter {
    Ixchel(IxchelFrontMatter),
    Legacy(LegacyFrontMatter),
}

#[derive(Debug)]
struct NormalizedFrontMatter {
    id: String,
    title: String,
    status: String,
    priority: u8,
    issue_type: String,
    created_at: String,
    updated_at: String,
    closed_at: Option<String>,
    created_by: String,
    created_by_type: String,
    assignee: Option<String>,
    agent_id: Option<String>,
    session_id: Option<String>,
    external_ref: Option<String>,
    parent: Option<String>,
    labels: Vec<String>,
    depends_on: Vec<DependencyFrontMatter>,
    estimated_minutes: Option<i32>,
}

impl AnyFrontMatter {
    fn normalize(self, path: &Path) -> Result<NormalizedFrontMatter> {
        match self {
            Self::Ixchel(front) => {
                if front.entity_type != "issue" {
                    return Err(HbdError::InvalidFormat {
                        path: path.to_path_buf(),
                        reason: format!("expected type: issue, got: {}", front.entity_type),
                    });
                }
                Ok(front.into())
            }
            Self::Legacy(front) => Ok(front.into()),
        }
    }
}

impl From<IxchelFrontMatter> for NormalizedFrontMatter {
    fn from(front: IxchelFrontMatter) -> Self {
        Self {
            id: front.id,
            title: front.title,
            status: front.status,
            priority: front.priority,
            issue_type: front.issue_type,
            created_at: front.created_at,
            updated_at: front.updated_at,
            closed_at: front.closed_at,
            created_by: front.created_by,
            created_by_type: front.created_by_type,
            assignee: front.assignee,
            agent_id: front.agent_id,
            session_id: front.session_id,
            external_ref: front.external_ref,
            parent: front.parent,
            labels: front.labels,
            depends_on: front.depends_on,
            estimated_minutes: front.estimated_minutes,
        }
    }
}

impl From<LegacyFrontMatter> for NormalizedFrontMatter {
    fn from(front: LegacyFrontMatter) -> Self {
        Self {
            id: front.id,
            title: front.title,
            status: front.status,
            priority: front.priority,
            issue_type: front.issue_type,
            created_at: front.created_at,
            updated_at: front.updated_at,
            closed_at: front.closed_at,
            created_by: front.created_by,
            created_by_type: front.created_by_type,
            assignee: front.assignee,
            agent_id: front.agent_id,
            session_id: front.session_id,
            external_ref: front.external_ref,
            parent: front.parent,
            labels: front.labels,
            depends_on: front.depends_on,
            estimated_minutes: front.estimated_minutes,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DependencyFrontMatter {
    id: String,
    #[serde(rename = "type", default = "default_dep_type")]
    dep_type: String,
}

fn default_dep_type() -> String {
    "blocks".to_string()
}

pub fn parse_issue(content: &str, path: &Path) -> Result<Issue> {
    let matter = Matter::<YAML>::new();
    let result = matter.parse(content);

    let front = parse_frontmatter(result.data, path)?;

    let (body, comments) = parse_body_and_comments(&result.content);
    build_issue(front, body, comments, path)
}

fn parse_frontmatter(data: Option<Pod>, path: &Path) -> Result<NormalizedFrontMatter> {
    let front: AnyFrontMatter = data
        .ok_or_else(|| HbdError::InvalidFormat {
            path: path.to_path_buf(),
            reason: "missing YAML frontmatter".to_string(),
        })?
        .deserialize()
        .map_err(|e| HbdError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("invalid frontmatter: {e}"),
        })?;

    front.normalize(path)
}

fn build_issue(
    front: NormalizedFrontMatter,
    body: String,
    comments: Vec<Comment>,
    path: &Path,
) -> Result<Issue> {
    let status: Status = front.status.parse().map_err(|e| HbdError::InvalidFormat {
        path: path.to_path_buf(),
        reason: e,
    })?;

    let priority = Priority::from_u8(front.priority).ok_or_else(|| HbdError::InvalidFormat {
        path: path.to_path_buf(),
        reason: format!("invalid priority: {}", front.priority),
    })?;

    let issue_type: IssueType = front
        .issue_type
        .parse()
        .map_err(|e| HbdError::InvalidFormat {
            path: path.to_path_buf(),
            reason: e,
        })?;

    let created_by_type = match front.created_by_type.as_str() {
        "agent" => CreatorType::Agent,
        _ => CreatorType::Human,
    };

    let depends_on = parse_dependencies(front.depends_on);

    Ok(Issue {
        id: front.id,
        title: front.title,
        body,
        status,
        priority,
        issue_type,
        created_at: parse_datetime(&front.created_at)?,
        updated_at: parse_datetime(&front.updated_at)?,
        closed_at: front.closed_at.as_deref().map(parse_datetime).transpose()?,
        created_by: front.created_by,
        created_by_type,
        assignee: front.assignee,
        agent_id: front.agent_id,
        session_id: front.session_id,
        external_ref: front.external_ref,
        parent_id: front.parent,
        labels: front.labels,
        depends_on,
        comments,
        estimated_minutes: front.estimated_minutes,
        content_hash: None,
    })
}

fn parse_dependencies(depends_on: Vec<DependencyFrontMatter>) -> Vec<Dependency> {
    depends_on
        .into_iter()
        .map(|d| Dependency {
            id: d.id,
            dep_type: d.dep_type.parse().unwrap_or(DepType::Blocks),
        })
        .collect()
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| HbdError::InvalidFrontmatter(format!("invalid datetime '{s}': {e}")))
}

fn parse_body_and_comments(content: &str) -> (String, Vec<Comment>) {
    let mut comments = Vec::new();
    let mut body_lines = Vec::new();
    let mut in_comments_section = false;
    let mut current_comment: Option<(DateTime<Utc>, String, CreatorType, Vec<String>)> = None;

    for line in content.lines() {
        if line.trim() == "## Comments" {
            in_comments_section = true;
            if let Some((ts, author, creator_type, lines)) = current_comment.take() {
                comments.push(build_comment(ts, author, creator_type, &lines));
            }
            continue;
        }

        if in_comments_section {
            if let Some(header) = line.strip_prefix("### ") {
                if let Some((ts, author, creator_type, lines)) = current_comment.take() {
                    comments.push(build_comment(ts, author, creator_type, &lines));
                }
                if let Some((ts, author, creator_type)) = parse_comment_header(header) {
                    current_comment = Some((ts, author, creator_type, Vec::new()));
                }
            } else if let Some((_, _, _, ref mut lines)) = current_comment {
                lines.push(line.to_string());
            }
        } else {
            body_lines.push(line);
        }
    }

    if let Some((ts, author, creator_type, lines)) = current_comment.take() {
        comments.push(build_comment(ts, author, creator_type, &lines));
    }

    let body = body_lines.join("\n").trim().to_string();
    (body, comments)
}

fn parse_comment_header(header: &str) -> Option<(DateTime<Utc>, String, CreatorType)> {
    let parts: Vec<&str> = header.splitn(2, " — ").collect();
    if parts.len() != 2 {
        return None;
    }

    let timestamp = DateTime::parse_from_rfc3339(parts[0].trim())
        .ok()?
        .with_timezone(&Utc);

    let author_part = parts[1].trim();
    let (author, creator_type) = if author_part.ends_with(" (agent)") {
        (
            author_part.trim_end_matches(" (agent)").to_string(),
            CreatorType::Agent,
        )
    } else if author_part.ends_with(" (human)") {
        (
            author_part.trim_end_matches(" (human)").to_string(),
            CreatorType::Human,
        )
    } else {
        (author_part.to_string(), CreatorType::Human)
    };

    Some((timestamp, author, creator_type))
}

fn build_comment(
    ts: DateTime<Utc>,
    author: String,
    creator_type: CreatorType,
    lines: &[String],
) -> Comment {
    Comment {
        id: crate::id::generate_comment_id(),
        body: lines.join("\n").trim().to_string(),
        created_at: ts,
        created_by: author,
        created_by_type: creator_type,
    }
}

pub fn render_issue(issue: &Issue) -> String {
    let front = IxchelFrontMatter {
        id: issue.id.clone(),
        title: issue.title.clone(),
        entity_type: "issue".to_string(),
        status: issue.status.as_str().to_string(),
        priority: issue.priority.as_u8(),
        issue_type: issue.issue_type.as_str().to_string(),
        created_at: issue.created_at.to_rfc3339(),
        updated_at: issue.updated_at.to_rfc3339(),
        closed_at: issue.closed_at.map(|dt| dt.to_rfc3339()),
        created_by: issue.created_by.clone(),
        created_by_type: issue.created_by_type.as_str().to_string(),
        assignee: issue.assignee.clone(),
        agent_id: issue.agent_id.clone(),
        session_id: issue.session_id.clone(),
        external_ref: issue.external_ref.clone(),
        parent: issue.parent_id.clone(),
        labels: issue.labels.clone(),
        depends_on: issue
            .depends_on
            .iter()
            .map(|d| DependencyFrontMatter {
                id: d.id.clone(),
                dep_type: d.dep_type.as_str().to_string(),
            })
            .collect(),
        estimated_minutes: issue.estimated_minutes,
    };

    let yaml = serde_yaml::to_string(&front).unwrap_or_default();
    let yaml = yaml.trim_start_matches("---\n").trim_end_matches('\n');

    let mut output = format!("---\n{yaml}\n---\n\n");

    if !issue.body.is_empty() {
        output.push_str(&issue.body);
        output.push('\n');
    }

    if !issue.comments.is_empty() {
        output.push_str("\n## Comments\n\n");
        for comment in &issue.comments {
            let creator_suffix = match comment.created_by_type {
                CreatorType::Agent => " (agent)",
                CreatorType::Human => " (human)",
            };
            let _ = write!(
                output,
                "### {} — {}{}\n\n",
                comment.created_at.to_rfc3339(),
                comment.created_by,
                creator_suffix
            );
            output.push_str(&comment.body);
            output.push_str("\n\n");
        }
    }

    output
}

pub fn compute_content_hash(issue: &Issue) -> String {
    let content = format!("{}\n{}", issue.title, issue.body);
    let hash = blake3::hash(content.as_bytes());
    hex::encode(&hash.as_bytes()[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let issue = Issue::builder("Test issue")
            .body("This is a test")
            .priority(Priority::High)
            .labels(["test", "mvp"])
            .build();

        let markdown = render_issue(&issue);
        let parsed = parse_issue(&markdown, Path::new("test.md")).unwrap();

        assert_eq!(parsed.id, issue.id);
        assert_eq!(parsed.title, issue.title);
        assert_eq!(parsed.body, issue.body);
        assert_eq!(parsed.priority, issue.priority);
        assert_eq!(parsed.labels, issue.labels);
    }

    #[test]
    fn test_parse_comments() {
        let content = r"---
id: bd-test01
title: Test
status: open
priority: 2
type: task
created_at: 2026-01-01T00:00:00Z
updated_at: 2026-01-01T00:00:00Z
created_by: kevin
---

Description here.

## Comments

### 2026-01-01T10:00:00Z — claude (agent)

First comment.

### 2026-01-01T11:00:00Z — kevin (human)

Second comment.
";

        let issue = parse_issue(content, Path::new("test.md")).unwrap();
        assert_eq!(issue.comments.len(), 2);
        assert_eq!(issue.comments[0].created_by, "claude");
        assert_eq!(issue.comments[0].created_by_type, CreatorType::Agent);
        assert_eq!(issue.comments[1].created_by, "kevin");
    }
}
