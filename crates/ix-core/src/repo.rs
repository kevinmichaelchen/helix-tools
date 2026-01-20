use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use ix_config::{ConfigLoader, IxchelConfig};
use serde_yaml::{Mapping, Value};
use thiserror::Error;

use crate::entity::{EntityKind, kind_from_id, looks_like_entity_id};
use crate::markdown::{
    MarkdownDocument, MarkdownError, get_string, get_string_list, parse_markdown, render_markdown,
    set_string, set_string_list,
};
use crate::paths::{IxchelPaths, find_git_root};

#[derive(Debug, Clone)]
pub struct EntitySummary {
    pub id: String,
    pub kind: EntityKind,
    pub title: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ListSort {
    #[default]
    CreatedDesc,
    UpdatedDesc,
}

#[derive(Debug, Error)]
pub enum ParseListSortError {
    #[error("Unknown sort option: {0}")]
    UnknownSort(String),
}

impl FromStr for ListSort {
    type Err = ParseListSortError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "recent" | "created" | "created_desc" | "created-desc" | "createddesc" => {
                Ok(Self::CreatedDesc)
            }
            "updated" | "updated_desc" | "updated-desc" | "updateddesc" => Ok(Self::UpdatedDesc),
            _ => Err(ParseListSortError::UnknownSort(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct CheckReport {
    pub errors: Vec<CheckError>,
}

#[derive(Debug)]
pub struct CheckError {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug)]
pub struct CheckIssue {
    pub path: PathBuf,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug)]
pub struct CheckReportDetailed {
    pub errors: Vec<CheckIssue>,
}

#[derive(Debug)]
pub struct IxchelRepo {
    pub paths: IxchelPaths,
    pub config: IxchelConfig,
}

const METADATA_KEYS: &[&str] = &[
    "id",
    "type",
    "title",
    "status",
    "date",
    "created_at",
    "updated_at",
    "created_by",
    "tags",
];

const KNOWN_ID_PREFIXES_HINT: &str = "dec, iss, bd, idea, rpt, src, cite, agt, ses";

impl IxchelRepo {
    pub fn open_from(start: &Path) -> Result<Self> {
        let repo_root = find_git_root(start).with_context(|| {
            format!(
                "Not inside a git repository (no .git found above {})",
                start.display()
            )
        })?;

        let paths = IxchelPaths::new(repo_root);
        let ixchel_dir = paths.ixchel_dir();
        if !ixchel_dir.exists() {
            anyhow::bail!(
                "Ixchel is not initialized (missing {}). Run `ixchel init`.",
                ixchel_dir.display()
            );
        }

        let config: IxchelConfig = ConfigLoader::new("").with_project_dir(ixchel_dir).load()?;

        Ok(Self { paths, config })
    }

    pub fn init_from(start: &Path, force: bool) -> Result<Self> {
        let repo_root = find_git_root(start).with_context(|| {
            format!(
                "Not inside a git repository (no .git found above {})",
                start.display()
            )
        })?;

        Self::init_at(&repo_root, force)
    }

    pub fn init_at(repo_root: &Path, force: bool) -> Result<Self> {
        let paths = IxchelPaths::new(repo_root.to_path_buf());
        let ixchel_dir = paths.ixchel_dir();

        if ixchel_dir.exists() && !force {
            anyhow::bail!(
                "{} already exists. Re-run with --force to recreate the directory layout.",
                ixchel_dir.display()
            );
        }

        std::fs::create_dir_all(&ixchel_dir)
            .with_context(|| format!("Failed to create {}", ixchel_dir.display()))?;
        paths.ensure_layout()?;
        ensure_project_gitignore(repo_root)?;

        let config_path = paths.config_path();
        if force || !config_path.exists() {
            IxchelConfig::default().save(&config_path)?;
        }

        let config: IxchelConfig = ConfigLoader::new("").with_project_dir(ixchel_dir).load()?;
        Ok(Self { paths, config })
    }

    pub fn create_entity(
        &self,
        kind: EntityKind,
        title: &str,
        status: Option<&str>,
    ) -> Result<EntitySummary> {
        let created_by = default_actor();
        let now = Utc::now();

        let id = ix_id::id_random(kind.id_prefix());
        let path = self.paths.kind_dir(kind).join(format!("{id}.md"));

        if path.exists() {
            anyhow::bail!("Entity already exists: {}", path.display());
        }

        let mut frontmatter = Mapping::new();
        frontmatter.insert(Value::String("id".to_string()), Value::String(id.clone()));
        frontmatter.insert(
            Value::String("type".to_string()),
            Value::String(kind.as_str().to_string()),
        );
        frontmatter.insert(
            Value::String("title".to_string()),
            Value::String(title.to_string()),
        );

        if let Some(status) = status {
            frontmatter.insert(
                Value::String("status".to_string()),
                Value::String(status.to_string()),
            );
        }

        frontmatter.insert(
            Value::String("created_at".to_string()),
            Value::String(now.to_rfc3339_opts(SecondsFormat::Secs, true)),
        );
        frontmatter.insert(
            Value::String("updated_at".to_string()),
            Value::String(now.to_rfc3339_opts(SecondsFormat::Secs, true)),
        );
        if let Some(created_by) = created_by {
            frontmatter.insert(
                Value::String("created_by".to_string()),
                Value::String(created_by),
            );
        }
        frontmatter.insert(
            Value::String("tags".to_string()),
            Value::Sequence(Vec::new()),
        );

        let body = default_template(kind);
        let doc = MarkdownDocument { frontmatter, body };
        let markdown = render_markdown(&doc)?;

        std::fs::write(&path, markdown)
            .with_context(|| format!("Failed to write {}", path.display()))?;

        Ok(EntitySummary {
            id,
            kind,
            title: title.to_string(),
            path,
        })
    }

    pub fn read_raw(&self, id: &str) -> Result<String> {
        let path = self
            .paths
            .entity_path(id)
            .with_context(|| format!("Unknown entity id prefix: {id}"))?;

        std::fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
    }

    pub fn delete_entity(&self, id: &str) -> Result<()> {
        let path = self
            .paths
            .entity_path(id)
            .with_context(|| format!("Unknown entity id prefix: {id}"))?;

        if !path.exists() {
            anyhow::bail!("Entity does not exist: {id} ({})", path.display());
        }

        std::fs::remove_file(&path)
            .with_context(|| format!("Failed to delete {}", path.display()))?;
        Ok(())
    }

    pub fn list(&self, kind: Option<EntityKind>, sort: ListSort) -> Result<Vec<EntitySummary>> {
        let mut out = Vec::new();

        let kinds: Vec<EntityKind> = kind.map_or_else(
            || {
                vec![
                    EntityKind::Decision,
                    EntityKind::Issue,
                    EntityKind::Idea,
                    EntityKind::Report,
                    EntityKind::Source,
                    EntityKind::Citation,
                    EntityKind::Agent,
                    EntityKind::Session,
                ]
            },
            |k| vec![k],
        );

        for kind in kinds {
            let dir = self.paths.kind_dir(kind);
            if !dir.exists() {
                continue;
            }

            for entry in std::fs::read_dir(&dir)
                .with_context(|| format!("Failed to read {}", dir.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }

                let raw = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read {}", path.display()))?;
                let doc = parse_markdown(&path, &raw)?;

                let id = get_string(&doc.frontmatter, "id")
                    .or_else(|| {
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .map(std::string::ToString::to_string)
                    })
                    .unwrap_or_default();
                let title = get_string(&doc.frontmatter, "title").unwrap_or_default();

                let summary = EntitySummary {
                    id,
                    kind,
                    title,
                    path,
                };

                let sort_ts = match sort {
                    ListSort::CreatedDesc => parse_timestamp(&doc.frontmatter, "created_at"),
                    ListSort::UpdatedDesc => parse_timestamp(&doc.frontmatter, "updated_at"),
                };

                out.push(ListEntry { summary, sort_ts });
            }
        }

        out.sort_by(|a, b| {
            cmp_timestamp_desc(
                a.sort_ts.as_ref(),
                b.sort_ts.as_ref(),
                &a.summary.id,
                &b.summary.id,
            )
        });

        Ok(out.into_iter().map(|entry| entry.summary).collect())
    }

    pub fn collect_tags(&self, kind: Option<EntityKind>) -> Result<HashMap<String, Vec<String>>> {
        let mut out: HashMap<String, Vec<String>> = HashMap::new();

        for item in self.list(kind, ListSort::default())? {
            let raw = std::fs::read_to_string(&item.path)
                .with_context(|| format!("Failed to read {}", item.path.display()))?;
            let doc = parse_markdown(&item.path, &raw)?;
            let tags = normalized_tags_vec(&doc.frontmatter);
            if tags.is_empty() {
                continue;
            }

            for tag in tags {
                out.entry(tag).or_default().push(item.id.clone());
            }
        }

        Ok(out)
    }

    pub fn list_untagged(&self, kind: Option<EntityKind>) -> Result<Vec<EntitySummary>> {
        let mut out = Vec::new();

        for item in self.list(kind, ListSort::default())? {
            let raw = std::fs::read_to_string(&item.path)
                .with_context(|| format!("Failed to read {}", item.path.display()))?;
            let doc = parse_markdown(&item.path, &raw)?;
            if normalized_tags_vec(&doc.frontmatter).is_empty() {
                out.push(item);
            }
        }

        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }

    pub fn add_tags(&self, id: &str, tags: &[String]) -> Result<bool> {
        let path = self
            .paths
            .entity_path(id)
            .with_context(|| format!("Unknown entity id prefix: {id}"))?;
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let mut doc = parse_markdown(&path, &raw)?;

        let mut existing = normalized_tags_vec(&doc.frontmatter);
        let mut changed = false;
        for tag in tags {
            let Some(tag) = normalize_tag(tag) else {
                continue;
            };
            if existing.iter().any(|value| value == &tag) {
                continue;
            }
            existing.push(tag);
            changed = true;
        }

        if !changed {
            return Ok(false);
        }

        set_string_list(&mut doc.frontmatter, "tags", existing);
        let now = Utc::now();
        set_string(
            &mut doc.frontmatter,
            "updated_at",
            now.to_rfc3339_opts(SecondsFormat::Secs, true),
        );

        let out = render_markdown(&doc)?;
        std::fs::write(&path, out)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(true)
    }

    pub fn remove_tags(&self, id: &str, tags: &[String]) -> Result<bool> {
        let path = self
            .paths
            .entity_path(id)
            .with_context(|| format!("Unknown entity id prefix: {id}"))?;
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let mut doc = parse_markdown(&path, &raw)?;

        let to_remove = tags
            .iter()
            .filter_map(|tag| normalize_tag(tag))
            .collect::<BTreeSet<_>>();
        if to_remove.is_empty() {
            return Ok(false);
        }

        let mut existing = normalized_tags_vec(&doc.frontmatter);
        let before_len = existing.len();
        existing.retain(|tag| !to_remove.contains(tag));

        if existing.len() == before_len {
            return Ok(false);
        }

        set_string_list(&mut doc.frontmatter, "tags", existing);
        let now = Utc::now();
        set_string(
            &mut doc.frontmatter,
            "updated_at",
            now.to_rfc3339_opts(SecondsFormat::Secs, true),
        );

        let out = render_markdown(&doc)?;
        std::fs::write(&path, out)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(true)
    }

    pub fn link(&self, from_id: &str, rel: &str, to_id: &str) -> Result<()> {
        let from_path = self
            .paths
            .entity_path(from_id)
            .with_context(|| format!("Unknown entity id prefix: {from_id}"))?;
        let to_path = self
            .paths
            .entity_path(to_id)
            .with_context(|| format!("Unknown entity id prefix: {to_id}"))?;

        if !to_path.exists() {
            anyhow::bail!("Target does not exist: {to_id} ({})", to_path.display());
        }

        let raw = std::fs::read_to_string(&from_path)
            .with_context(|| format!("Failed to read {}", from_path.display()))?;
        let mut doc = parse_markdown(&from_path, &raw)?;

        let mut values = get_string_list(&doc.frontmatter, rel);
        if !values.iter().any(|v| v == to_id) {
            values.push(to_id.to_string());
        }
        set_string_list(&mut doc.frontmatter, rel, values);

        let now = Utc::now();
        set_string(
            &mut doc.frontmatter,
            "updated_at",
            now.to_rfc3339_opts(SecondsFormat::Secs, true),
        );

        let out = render_markdown(&doc)?;
        std::fs::write(&from_path, out)
            .with_context(|| format!("Failed to write {}", from_path.display()))?;
        Ok(())
    }

    pub fn unlink(&self, from_id: &str, rel: &str, to_id: &str) -> Result<bool> {
        let from_path = self
            .paths
            .entity_path(from_id)
            .with_context(|| format!("Unknown entity id prefix: {from_id}"))?;

        let raw = std::fs::read_to_string(&from_path)
            .with_context(|| format!("Failed to read {}", from_path.display()))?;
        let mut doc = parse_markdown(&from_path, &raw)?;

        let mut values = get_string_list(&doc.frontmatter, rel);
        let before_len = values.len();
        values.retain(|v| v != to_id);

        if values.len() == before_len {
            return Ok(false);
        }

        if values.is_empty() {
            doc.frontmatter.remove(Value::String(rel.to_string()));
        } else {
            set_string_list(&mut doc.frontmatter, rel, values);
        }

        let now = Utc::now();
        set_string(
            &mut doc.frontmatter,
            "updated_at",
            now.to_rfc3339_opts(SecondsFormat::Secs, true),
        );

        let out = render_markdown(&doc)?;
        std::fs::write(&from_path, out)
            .with_context(|| format!("Failed to write {}", from_path.display()))?;

        Ok(true)
    }

    pub fn check(&self) -> Result<CheckReport> {
        let report = self.check_with_suggestions()?;
        Ok(CheckReport {
            errors: report
                .errors
                .into_iter()
                .map(|issue| CheckError {
                    path: issue.path,
                    message: issue.message,
                })
                .collect(),
        })
    }

    pub fn check_with_suggestions(&self) -> Result<CheckReportDetailed> {
        let mut errors = Vec::new();
        let mut seen_ids: BTreeSet<String> = BTreeSet::new();

        let kinds = [
            EntityKind::Decision,
            EntityKind::Issue,
            EntityKind::Idea,
            EntityKind::Report,
            EntityKind::Source,
            EntityKind::Citation,
            EntityKind::Agent,
            EntityKind::Session,
        ];

        for kind in kinds {
            let dir = self.paths.kind_dir(kind);
            if !dir.exists() {
                continue;
            }

            let mut entries = Vec::new();
            for entry in std::fs::read_dir(&dir)
                .with_context(|| format!("Failed to read {}", dir.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                entries.push(path);
            }

            entries.sort();
            for path in entries {
                check_document(&self.paths, kind, &path, &mut seen_ids, &mut errors)?;
            }
        }

        Ok(CheckReportDetailed { errors })
    }
}

fn check_document(
    paths: &IxchelPaths,
    kind: EntityKind,
    path: &Path,
    seen_ids: &mut BTreeSet<String>,
    errors: &mut Vec<CheckIssue>,
) -> Result<()> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let file_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    let has_frontmatter = raw.lines().next().is_some_and(|line| line == "---");

    let doc = parse_document_with_issue(path, &raw, errors);
    if !has_frontmatter {
        let id_hint = id_hint(&file_id, kind);
        let suggestion = format!(
            "Add YAML frontmatter starting with `---` and include `id: {id_hint}`, `type: {}`, `title`, `created_at`, `updated_at`, and `tags`.",
            kind.as_str()
        );
        push_issue(errors, path, "missing frontmatter block", Some(suggestion));
    }

    let frontmatter = if has_frontmatter {
        doc.as_ref().map(|doc| &doc.frontmatter)
    } else {
        None
    };

    let frontmatter_id = frontmatter
        .and_then(|frontmatter| check_frontmatter_id(frontmatter, &file_id, kind, path, errors));
    let resolved_id = frontmatter_id.as_deref().unwrap_or(file_id.as_str());
    check_id_and_path(resolved_id, kind, path, seen_ids, errors);

    if let Some(frontmatter) = frontmatter {
        check_frontmatter_fields(frontmatter, kind, path, errors);
        check_relationships(paths, frontmatter, path, errors);
    }

    Ok(())
}

fn parse_document_with_issue(
    path: &Path,
    raw: &str,
    errors: &mut Vec<CheckIssue>,
) -> Option<MarkdownDocument> {
    match parse_markdown(path, raw) {
        Ok(doc) => Some(doc),
        Err(err) => {
            let (message, suggestion) = match err {
                MarkdownError::UnclosedFrontmatter { .. } => (
                    "frontmatter missing closing delimiter '---'".to_string(),
                    "Add a closing `---` line after the YAML frontmatter.".to_string(),
                ),
                MarkdownError::FrontmatterParse { source, .. } => (
                    format!("frontmatter YAML parse error: {source}"),
                    "Fix YAML syntax; frontmatter must be a mapping of key/value pairs."
                        .to_string(),
                ),
                MarkdownError::FrontmatterNotMapping { .. } => (
                    "frontmatter must be a YAML mapping".to_string(),
                    "Replace frontmatter with key/value mapping (for example: `id: ...`)."
                        .to_string(),
                ),
                MarkdownError::FrontmatterSerialize { source } => (
                    format!("frontmatter serialization error: {source}"),
                    "Fix frontmatter values so they can be serialized to YAML.".to_string(),
                ),
            };
            push_issue(errors, path, message, Some(suggestion));
            None
        }
    }
}

fn check_frontmatter_id(
    frontmatter: &Mapping,
    file_id: &str,
    kind: EntityKind,
    path: &Path,
    errors: &mut Vec<CheckIssue>,
) -> Option<String> {
    match frontmatter.get(Value::String("id".to_string())) {
        Some(Value::String(value)) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                let id_hint = id_hint(file_id, kind);
                push_issue(
                    errors,
                    path,
                    "missing frontmatter id",
                    Some(format!("Add `id: {id_hint}` to frontmatter.")),
                );
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Some(_) => {
            push_issue(
                errors,
                path,
                "frontmatter id must be a string",
                Some("Set `id` to a string like `iss-a1b2c3`.".to_string()),
            );
            None
        }
        None => {
            let id_hint = id_hint(file_id, kind);
            push_issue(
                errors,
                path,
                "missing frontmatter id",
                Some(format!("Add `id: {id_hint}` to frontmatter.")),
            );
            None
        }
    }
}

fn check_id_and_path(
    id: &str,
    kind: EntityKind,
    path: &Path,
    seen_ids: &mut BTreeSet<String>,
    errors: &mut Vec<CheckIssue>,
) {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return;
    }

    let mut id_format_ok = true;
    if ix_id::parse_id(trimmed).is_err() {
        id_format_ok = false;
        push_issue(
            errors,
            path,
            "id is not a valid Ixchel id",
            Some("Use `<prefix>-<6..12 hex>`, for example `iss-a1b2c3`.".to_string()),
        );
    }

    if !seen_ids.insert(trimmed.to_string()) {
        push_issue(
            errors,
            path,
            format!("duplicate id: {trimmed}"),
            Some("Make ids unique and rename the file to match the new id.".to_string()),
        );
    }

    if id_format_ok {
        let expected_kind = kind_from_id(trimmed);
        if expected_kind != Some(kind) {
            let suggestion = if expected_kind.is_none() {
                format!(
                    "Use a known id prefix ({KNOWN_ID_PREFIXES_HINT}) or move the file to the correct directory.",
                )
            } else {
                format!(
                    "Move the file to `{}` or update the id prefix to `{}`.",
                    kind.directory_name(),
                    kind.id_prefix()
                )
            };
            push_issue(
                errors,
                path,
                format!(
                    "id prefix does not match directory (id={trimmed}, dir={})",
                    kind.directory_name()
                ),
                Some(suggestion),
            );
        }
    }

    let expected_file = format!("{trimmed}.md");
    if path
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|name| name != expected_file)
    {
        push_issue(
            errors,
            path,
            format!("file name does not match id (expected {expected_file})"),
            Some(format!(
                "Rename the file to `{expected_file}` or update `id` to match the filename.",
            )),
        );
    }
}

fn check_frontmatter_fields(
    frontmatter: &Mapping,
    kind: EntityKind,
    path: &Path,
    errors: &mut Vec<CheckIssue>,
) {
    check_frontmatter_type(frontmatter, kind, path, errors);
    check_frontmatter_title(frontmatter, path, errors);
    check_timestamp(frontmatter, "created_at", path, errors);
    check_timestamp(frontmatter, "updated_at", path, errors);
    check_tags_field(frontmatter, path, errors);
    check_optional_string_field(frontmatter, "status", path, errors);
    check_optional_string_field(frontmatter, "created_by", path, errors);
    check_optional_string_field(frontmatter, "date", path, errors);
}

fn check_frontmatter_type(
    frontmatter: &Mapping,
    kind: EntityKind,
    path: &Path,
    errors: &mut Vec<CheckIssue>,
) {
    match frontmatter.get(Value::String("type".to_string())) {
        Some(Value::String(value)) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                push_issue(
                    errors,
                    path,
                    "missing frontmatter type",
                    Some(format!("Add `type: {}` to frontmatter.", kind.as_str())),
                );
                return;
            }

            match trimmed.parse::<EntityKind>() {
                Ok(parsed) => {
                    if parsed != kind {
                        push_issue(
                            errors,
                            path,
                            format!(
                                "frontmatter type does not match directory (type={trimmed}, dir={})",
                                kind.directory_name()
                            ),
                            Some(format!(
                                "Set `type` to `{}` or move the file to the correct directory.",
                                kind.as_str()
                            )),
                        );
                    }
                }
                Err(_) => {
                    push_issue(
                        errors,
                        path,
                        format!("unknown frontmatter type: {trimmed}"),
                        Some(format!(
                            "Set `type` to `{}` or move the file to the correct directory.",
                            kind.as_str()
                        )),
                    );
                }
            }
        }
        Some(_) => {
            push_issue(
                errors,
                path,
                "frontmatter type must be a string",
                Some("Set `type` to a string like `issue`.".to_string()),
            );
        }
        None => {
            push_issue(
                errors,
                path,
                "missing frontmatter type",
                Some(format!("Add `type: {}` to frontmatter.", kind.as_str())),
            );
        }
    }
}

fn check_frontmatter_title(frontmatter: &Mapping, path: &Path, errors: &mut Vec<CheckIssue>) {
    match frontmatter.get(Value::String("title".to_string())) {
        Some(Value::String(value)) => {
            if value.trim().is_empty() {
                push_issue(
                    errors,
                    path,
                    "missing or empty title",
                    Some("Add a non-empty `title` string.".to_string()),
                );
            }
        }
        Some(_) => {
            push_issue(
                errors,
                path,
                "frontmatter title must be a string",
                Some("Set `title` to a string value.".to_string()),
            );
        }
        None => {
            push_issue(
                errors,
                path,
                "missing or empty title",
                Some("Add a non-empty `title` string.".to_string()),
            );
        }
    }
}

fn check_timestamp(frontmatter: &Mapping, key: &str, path: &Path, errors: &mut Vec<CheckIssue>) {
    match frontmatter.get(Value::String(key.to_string())) {
        Some(Value::String(value)) => {
            if DateTime::parse_from_rfc3339(value).is_err() {
                push_issue(
                    errors,
                    path,
                    format!("{key} is not RFC3339"),
                    Some(format!(
                        "Set `{key}` to an RFC3339 timestamp, for example `2024-01-01T00:00:00Z`.",
                    )),
                );
            }
        }
        Some(_) => {
            push_issue(
                errors,
                path,
                format!("{key} must be a string"),
                Some(format!("Set `{key}` to an RFC3339 string.")),
            );
        }
        None => {
            push_issue(
                errors,
                path,
                format!("missing {key} timestamp"),
                Some(format!(
                    "Add `{key}` in RFC3339, for example `2024-01-01T00:00:00Z`.",
                )),
            );
        }
    }
}

fn check_tags_field(frontmatter: &Mapping, path: &Path, errors: &mut Vec<CheckIssue>) {
    let Some(value) = frontmatter.get(Value::String("tags".to_string())) else {
        return;
    };

    let tags_ok = match value {
        Value::Sequence(seq) => seq.iter().all(|item| matches!(item, Value::String(_))),
        Value::String(_) => true,
        _ => false,
    };
    if !tags_ok {
        push_issue(
            errors,
            path,
            "tags must be a string or list of strings",
            Some("Use `tags: []` or `tags: [\"foo\", \"bar\"]`.".to_string()),
        );
    }
}

fn check_optional_string_field(
    frontmatter: &Mapping,
    key: &str,
    path: &Path,
    errors: &mut Vec<CheckIssue>,
) {
    if let Some(value) = frontmatter.get(Value::String(key.to_string()))
        && !matches!(value, Value::String(_))
    {
        push_issue(
            errors,
            path,
            format!("{key} must be a string"),
            Some(format!("Set `{key}` to a string.")),
        );
    }
}

fn check_relationships(
    paths: &IxchelPaths,
    frontmatter: &Mapping,
    path: &Path,
    errors: &mut Vec<CheckIssue>,
) {
    for (rel, targets) in extract_relationships(frontmatter) {
        for target in targets {
            let Some(target_path) = paths.entity_path(&target) else {
                push_issue(
                    errors,
                    path,
                    format!("unknown id prefix in {rel}: {target}"),
                    Some(format!(
                        "Use a known id prefix ({KNOWN_ID_PREFIXES_HINT}) in `{rel}`.",
                    )),
                );
                continue;
            };

            if !target_path.exists() {
                let suggestion = format!(
                    "Create `{}` or remove `{rel}` -> `{target}`.",
                    target_path.display()
                );
                push_issue(
                    errors,
                    path,
                    format!("broken link {rel} -> {target}"),
                    Some(suggestion),
                );
            }
        }
    }
}

fn push_issue(
    errors: &mut Vec<CheckIssue>,
    path: &Path,
    message: impl Into<String>,
    suggestion: Option<String>,
) {
    errors.push(CheckIssue {
        path: path.to_path_buf(),
        message: message.into(),
        suggestion,
    });
}

fn parse_timestamp(frontmatter: &Mapping, key: &str) -> Option<DateTime<Utc>> {
    let raw = get_string(frontmatter, key)?;
    let parsed = DateTime::parse_from_rfc3339(&raw).ok()?;
    Some(parsed.with_timezone(&Utc))
}

fn cmp_timestamp_desc(
    a: Option<&DateTime<Utc>>,
    b: Option<&DateTime<Utc>>,
    a_id: &str,
    b_id: &str,
) -> Ordering {
    match (a, b) {
        (Some(a_ts), Some(b_ts)) => b_ts.cmp(a_ts).then_with(|| a_id.cmp(b_id)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a_id.cmp(b_id),
    }
}

struct ListEntry {
    summary: EntitySummary,
    sort_ts: Option<DateTime<Utc>>,
}

fn default_actor() -> Option<String> {
    std::env::var("IXCHEL_ACTOR")
        .ok()
        .or_else(|| std::env::var("USER").ok())
        .or_else(|| std::env::var("USERNAME").ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn default_template(kind: EntityKind) -> String {
    match kind {
        EntityKind::Decision => "## Context\n\n_Why is this decision needed?_\n\n## Decision\n\n_What did we decide?_\n\n## Consequences\n\n_What are the implications?_\n".to_string(),
        EntityKind::Issue => "## Problem\n\n_What is broken or missing?_\n\n## Plan\n\n- [ ] _Add steps_\n".to_string(),
        EntityKind::Idea => "## Summary\n\n_Describe the idea._\n".to_string(),
        EntityKind::Report => "## Summary\n\n_What did we learn?_\n".to_string(),
        EntityKind::Source => "## Summary\n\n_What is this source?_\n".to_string(),
        EntityKind::Citation => "## Quote\n\n> _Paste the quote here._\n".to_string(),
        EntityKind::Agent => "## Notes\n\n_Agent description and preferences._\n".to_string(),
        EntityKind::Session => "## Notes\n\n_Session context._\n".to_string(),
    }
}

fn ensure_project_gitignore(repo_root: &Path) -> Result<()> {
    let path = repo_root.join(".gitignore");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();

    let has_data = existing.lines().any(|l| l.trim() == ".ixchel/data/");
    let has_models = existing.lines().any(|l| l.trim() == ".ixchel/models/");
    if has_data && has_models {
        return Ok(());
    }

    let mut out = existing;
    if !out.ends_with('\n') && !out.is_empty() {
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }

    out.push_str("# Ixchel (rebuildable cache)\n");
    if !has_data {
        out.push_str(".ixchel/data/\n");
    }
    if !has_models {
        out.push_str(".ixchel/models/\n");
    }

    std::fs::write(&path, out).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn extract_relationships(frontmatter: &serde_yaml::Mapping) -> Vec<(String, Vec<String>)> {
    let mut rels = Vec::new();

    for (key, value) in frontmatter {
        let serde_yaml::Value::String(key) = key else {
            continue;
        };

        if METADATA_KEYS.contains(&key.as_str()) {
            continue;
        }

        let targets = match value {
            serde_yaml::Value::Sequence(seq) => seq
                .iter()
                .filter_map(|v| match v {
                    serde_yaml::Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            serde_yaml::Value::String(s) => vec![s.clone()],
            _ => Vec::new(),
        };

        let targets = targets
            .into_iter()
            .filter(|t| looks_like_entity_id(t))
            .collect::<Vec<_>>();

        if targets.is_empty() {
            continue;
        }

        rels.push((key.clone(), targets));
    }

    rels
}

fn normalize_tag(tag: &str) -> Option<String> {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalized_tags_vec(frontmatter: &Mapping) -> Vec<String> {
    let mut tags = Vec::new();
    let mut seen = BTreeSet::new();
    for tag in get_string_list(frontmatter, "tags") {
        if let Some(tag) = normalize_tag(&tag)
            && seen.insert(tag.clone())
        {
            tags.push(tag);
        }
    }
    tags
}

fn id_hint(file_id: &str, kind: EntityKind) -> String {
    let trimmed = file_id.trim();
    if trimmed.is_empty() {
        return format!("{}-<hash>", kind.id_prefix());
    }

    if ix_id::parse_id(trimmed).is_ok() {
        trimmed.to_string()
    } else {
        format!("{}-<hash>", kind.id_prefix())
    }
}
