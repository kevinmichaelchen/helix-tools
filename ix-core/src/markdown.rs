use std::path::{Path, PathBuf};

use serde_yaml::{Mapping, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MarkdownError {
    #[error("Missing closing frontmatter delimiter '---' in {path}")]
    UnclosedFrontmatter { path: PathBuf },

    #[error("Frontmatter in {path} must be a YAML mapping")]
    FrontmatterNotMapping { path: PathBuf },

    #[error("Failed to parse YAML frontmatter in {path}: {source}")]
    FrontmatterParse {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("Failed to serialize YAML frontmatter: {source}")]
    FrontmatterSerialize {
        #[source]
        source: serde_yaml::Error,
    },
}

#[derive(Debug, Clone)]
pub struct MarkdownDocument {
    pub frontmatter: Mapping,
    pub body: String,
}

pub fn parse_markdown(path: &Path, contents: &str) -> Result<MarkdownDocument, MarkdownError> {
    let mut lines = contents.lines();
    let Some(first_line) = lines.next() else {
        return Ok(MarkdownDocument {
            frontmatter: Mapping::new(),
            body: String::new(),
        });
    };

    if first_line != "---" {
        return Ok(MarkdownDocument {
            frontmatter: Mapping::new(),
            body: contents.to_string(),
        });
    }

    let mut yaml = String::new();
    let mut found_end = false;

    for line in lines.by_ref() {
        if line == "---" {
            found_end = true;
            break;
        }
        yaml.push_str(line);
        yaml.push('\n');
    }

    if !found_end {
        return Err(MarkdownError::UnclosedFrontmatter {
            path: path.to_path_buf(),
        });
    }

    let value: Value =
        serde_yaml::from_str(&yaml).map_err(|source| MarkdownError::FrontmatterParse {
            path: path.to_path_buf(),
            source,
        })?;

    let Value::Mapping(frontmatter) = value else {
        return Err(MarkdownError::FrontmatterNotMapping {
            path: path.to_path_buf(),
        });
    };

    let body = lines.collect::<Vec<_>>().join("\n");
    Ok(MarkdownDocument { frontmatter, body })
}

pub fn render_markdown(doc: &MarkdownDocument) -> Result<String, MarkdownError> {
    let mut out = String::new();
    out.push_str("---\n");

    let yaml = serde_yaml::to_string(&Value::Mapping(doc.frontmatter.clone()))
        .map_err(|source| MarkdownError::FrontmatterSerialize { source })?;
    let yaml = yaml.strip_prefix("---\n").unwrap_or(&yaml);
    out.push_str(yaml);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("---\n\n");

    out.push_str(&doc.body);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

#[must_use]
pub fn get_string(frontmatter: &Mapping, key: &str) -> Option<String> {
    frontmatter
        .get(&Value::String(key.to_string()))
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
}

pub fn set_string(frontmatter: &mut Mapping, key: &str, value: impl Into<String>) {
    frontmatter.insert(Value::String(key.to_string()), Value::String(value.into()));
}

#[must_use]
pub fn get_string_list(frontmatter: &Mapping, key: &str) -> Vec<String> {
    let Some(value) = frontmatter.get(&Value::String(key.to_string())) else {
        return Vec::new();
    };

    match value {
        Value::Sequence(seq) => seq
            .iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        Value::String(s) => vec![s.clone()],
        _ => Vec::new(),
    }
}

pub fn set_string_list(frontmatter: &mut Mapping, key: &str, values: Vec<String>) {
    let seq = values.into_iter().map(Value::String).collect();
    frontmatter.insert(Value::String(key.to_string()), Value::Sequence(seq));
}
