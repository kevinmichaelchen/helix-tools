use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use ix_config::{ConfigLoader, IxchelConfig};
use serde_yaml::{Mapping, Value};

use crate::entity::{EntityKind, kind_from_id, looks_like_entity_id};
use crate::markdown::{
    MarkdownDocument, get_string, get_string_list, parse_markdown, render_markdown, set_string,
    set_string_list,
};
use crate::paths::{IxchelPaths, find_git_root};

#[derive(Debug, Clone)]
pub struct EntitySummary {
    pub id: String,
    pub kind: EntityKind,
    pub title: String,
    pub path: PathBuf,
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

    pub fn list(&self, kind: Option<EntityKind>) -> Result<Vec<EntitySummary>> {
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

                out.push(EntitySummary {
                    id,
                    kind,
                    title,
                    path,
                });
            }
        }

        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }

    pub fn collect_tags(&self, kind: Option<EntityKind>) -> Result<HashMap<String, Vec<String>>> {
        let mut out: HashMap<String, Vec<String>> = HashMap::new();

        for item in self.list(kind)? {
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

        for item in self.list(kind)? {
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
        let mut errors = Vec::new();
        let mut seen_ids: BTreeSet<String> = BTreeSet::new();

        for item in self.list(None)? {
            if item.id.trim().is_empty() {
                errors.push(CheckError {
                    path: item.path.clone(),
                    message: "missing frontmatter id".to_string(),
                });
                continue;
            }

            if !seen_ids.insert(item.id.clone()) {
                errors.push(CheckError {
                    path: item.path.clone(),
                    message: format!("duplicate id: {}", item.id),
                });
            }

            let expected_kind = kind_from_id(&item.id);
            if expected_kind != Some(item.kind) {
                errors.push(CheckError {
                    path: item.path.clone(),
                    message: format!(
                        "id prefix does not match directory (id={}, dir={})",
                        item.id,
                        item.kind.directory_name()
                    ),
                });
            }

            if item.title.trim().is_empty() {
                errors.push(CheckError {
                    path: item.path.clone(),
                    message: "missing or empty title".to_string(),
                });
            }

            let expected_file = format!("{}.md", item.id);
            if item
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|name| name != expected_file)
            {
                errors.push(CheckError {
                    path: item.path.clone(),
                    message: format!("file name does not match id (expected {expected_file})"),
                });
            }

            let raw = std::fs::read_to_string(&item.path)
                .with_context(|| format!("Failed to read {}", item.path.display()))?;
            let doc = parse_markdown(&item.path, &raw)?;

            for (rel, targets) in extract_relationships(&doc.frontmatter) {
                for target in targets {
                    let Some(target_path) = self.paths.entity_path(&target) else {
                        errors.push(CheckError {
                            path: item.path.clone(),
                            message: format!("unknown id prefix in {rel}: {target}"),
                        });
                        continue;
                    };

                    if !target_path.exists() {
                        errors.push(CheckError {
                            path: item.path.clone(),
                            message: format!("broken link {rel} -> {target}"),
                        });
                    }
                }
            }
        }

        Ok(CheckReport { errors })
    }
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
