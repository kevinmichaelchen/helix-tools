use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use serde_yaml::{Mapping, Value as YamlValue};

use crate::entity::EntityKind;
use crate::markdown::{MarkdownDocument, render_markdown, set_string};
use crate::paths::IXCHEL_DIR_NAME;
use crate::repo::IxchelRepo;

#[derive(Debug, Clone)]
pub struct DecisionsMigrationReport {
    pub scanned: u32,
    pub created: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone)]
pub struct MigrateDecisionsOptions {
    pub source_dir: PathBuf,
    pub force: bool,
    pub dry_run: bool,
}

pub fn migrate_decisions(
    repo: &IxchelRepo,
    options: &MigrateDecisionsOptions,
) -> Result<DecisionsMigrationReport> {
    let source_dir = if options.source_dir.is_absolute() {
        options.source_dir.clone()
    } else {
        repo.paths.repo_root().join(&options.source_dir)
    };

    if !source_dir.exists() {
        anyhow::bail!(
            "Source decisions directory does not exist: {}",
            source_dir.display()
        );
    }

    let mut report = DecisionsMigrationReport {
        scanned: 0,
        created: 0,
        skipped: 0,
    };

    let mut paths = std::fs::read_dir(&source_dir)
        .with_context(|| format!("Failed to read {}", source_dir.display()))?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::result::Result<Vec<_>, std::io::Error>>()?;
    paths.sort();

    for path in paths {
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        report.scanned += 1;

        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let (title, status, date, deciders, tags) = parse_adr_metadata(&raw);

        let relative = path.strip_prefix(repo.paths.repo_root()).unwrap_or(&path);
        let id = helix_id::id_from_key("dec", &relative.to_string_lossy());
        let target_path = repo
            .paths
            .ixchel_dir()
            .join(EntityKind::Decision.directory_name())
            .join(format!("{id}.md"));

        if target_path.exists() && !options.force {
            report.skipped += 1;
            continue;
        }

        let now = Utc::now();

        let mut frontmatter = Mapping::new();
        frontmatter.insert(
            YamlValue::String("id".to_string()),
            YamlValue::String(id.clone()),
        );
        frontmatter.insert(
            YamlValue::String("type".to_string()),
            YamlValue::String(EntityKind::Decision.as_str().to_string()),
        );
        frontmatter.insert(
            YamlValue::String("title".to_string()),
            YamlValue::String(title.unwrap_or_else(|| id.clone())),
        );

        if let Some(status) = status {
            frontmatter.insert(
                YamlValue::String("status".to_string()),
                YamlValue::String(status),
            );
        }

        if let Some(date) = date {
            frontmatter.insert(
                YamlValue::String("date".to_string()),
                YamlValue::String(date),
            );
        }

        set_string(
            &mut frontmatter,
            "created_at",
            now.to_rfc3339_opts(SecondsFormat::Secs, true),
        );
        set_string(
            &mut frontmatter,
            "updated_at",
            now.to_rfc3339_opts(SecondsFormat::Secs, true),
        );

        if let Some(first_decider) = deciders.first().cloned() {
            set_string(&mut frontmatter, "created_by", first_decider);
        }

        let tags = tags
            .into_iter()
            .map(|t| YamlValue::String(t))
            .collect::<Vec<_>>();
        frontmatter.insert(
            YamlValue::String("tags".to_string()),
            YamlValue::Sequence(tags),
        );

        let body = format!(
            "> Migrated from `{}` into `{IXCHEL_DIR_NAME}/decisions/`.\n\n{raw}\n",
            relative.to_string_lossy()
        );

        let doc = MarkdownDocument { frontmatter, body };
        let rendered = render_markdown(&doc)?;

        if options.dry_run {
            report.created += 1;
            continue;
        }

        std::fs::write(&target_path, rendered)
            .with_context(|| format!("Failed to write {}", target_path.display()))?;
        report.created += 1;
    }

    Ok(report)
}

fn parse_adr_metadata(
    raw: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Vec<String>,
    Vec<String>,
) {
    let mut title: Option<String> = None;
    let mut status: Option<String> = None;
    let mut date: Option<String> = None;
    let mut deciders: Vec<String> = Vec::new();
    let mut tags: Vec<String> = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();

        if title.is_none() {
            if let Some(rest) = trimmed.strip_prefix("# ") {
                title = Some(rest.trim().to_string());
            }
        }

        if status.is_none() {
            if let Some(value) = parse_adr_kv(trimmed, "Status") {
                status = Some(value.to_ascii_lowercase());
                continue;
            }
        }

        if date.is_none() {
            if let Some(value) = parse_adr_kv(trimmed, "Date") {
                date = Some(value);
                continue;
            }
        }

        if deciders.is_empty() {
            if let Some(value) = parse_adr_kv(trimmed, "Deciders") {
                deciders = split_csv(&value);
                continue;
            }
        }

        if tags.is_empty() {
            if let Some(value) = parse_adr_kv(trimmed, "Tags") {
                tags = split_csv(&value);
                continue;
            }
        }
    }

    (title, status, date, deciders, tags)
}

fn parse_adr_kv(line: &str, key: &str) -> Option<String> {
    let prefix = format!("**{key}:**");
    let rest = line.strip_prefix(&prefix)?;
    let rest = rest.trim().trim_end_matches('\\').trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}
