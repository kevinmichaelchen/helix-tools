use crate::model::Language;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub language: Language,
}

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub root: PathBuf,
    pub include_hidden: bool,
    pub languages: Vec<LanguageConfig>,
}

#[derive(Debug, Clone)]
pub struct LanguageConfig {
    pub extension: &'static str,
    pub language: Language,
}

impl ScanConfig {
    pub fn rust_default(root: PathBuf) -> Self {
        Self {
            root,
            include_hidden: false,
            languages: vec![LanguageConfig {
                extension: "rs",
                language: Language::Rust,
            }],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scanner {
    config: ScanConfig,
}

impl Scanner {
    pub fn new(config: ScanConfig) -> Self {
        Self { config }
    }

    pub fn scan(&self) -> Result<Vec<SourceFile>> {
        let mut builder = WalkBuilder::new(&self.config.root);
        builder
            .hidden(!self.config.include_hidden)
            .ignore(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .filter_entry(|entry| !is_ignored_dir(entry.path()));

        let mut files = Vec::new();
        for result in builder.build() {
            let entry = result.context("failed to read directory entry")?;
            let file_type = entry.file_type();
            if !file_type.map(|kind| kind.is_file()).unwrap_or(false) {
                continue;
            }

            let path = entry.path();
            let language = match language_for_path(&self.config.languages, path) {
                Some(language) => language,
                None => continue,
            };

            let relative = path
                .strip_prefix(&self.config.root)
                .unwrap_or(path)
                .to_path_buf();
            files.push(SourceFile {
                path: relative,
                language,
            });
        }

        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }
}

fn language_for_path(configs: &[LanguageConfig], path: &Path) -> Option<Language> {
    let extension = path.extension()?.to_str()?;
    configs
        .iter()
        .find(|config| config.extension.eq_ignore_ascii_case(extension))
        .map(|config| config.language)
}

fn is_ignored_dir(path: &Path) -> bool {
    let name = path.file_name().and_then(|value| value.to_str());
    matches!(
        name,
        Some(".git" | "target" | "node_modules" | ".helix-map")
    )
}
