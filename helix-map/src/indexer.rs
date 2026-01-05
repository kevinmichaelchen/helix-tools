use crate::extract::SymbolExtractor;
use crate::model::{FileIndex, Index};
use crate::scanner::{ScanConfig, Scanner, SourceFile};
use crate::storage::IndexStore;
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;

pub struct Indexer<'a, S, E> {
    store: &'a S,
    extractor: &'a E,
}

impl<'a, S, E> Indexer<'a, S, E>
where
    S: IndexStore,
    E: SymbolExtractor,
{
    pub fn new(store: &'a S, extractor: &'a E) -> Self {
        Self { store, extractor }
    }

    pub fn index(&self, root: &Path) -> Result<Index> {
        let scan_config = ScanConfig::rust_default(root.to_path_buf());
        let scanner = Scanner::new(scan_config);
        let sources = scanner.scan()?;

        let previous = self.store.load()?;
        let mut previous_files = HashMap::new();
        if let Some(index) = previous {
            for file in index.files {
                previous_files.insert(file.path.clone(), file);
            }
        }

        let mut files = Vec::new();
        for source in sources {
            files.push(self.index_file(root, &source, &previous_files)?);
        }

        let index = Index {
            version: 1,
            root: root.to_path_buf(),
            generated_at: Utc::now(),
            files,
        };

        self.store.save(&index)?;
        Ok(index)
    }

    fn index_file(
        &self,
        root: &Path,
        source: &SourceFile,
        previous: &HashMap<PathBuf, FileIndex>,
    ) -> Result<FileIndex> {
        let absolute = root.join(&source.path);
        let bytes = fs::read(&absolute)
            .with_context(|| format!("failed to read {}", absolute.display()))?;
        let hash = file_hash(&bytes);

        if let Some(prev) = previous.get(&source.path)
            && prev.hash == hash
        {
            return Ok(prev.clone());
        }

        let source_text = String::from_utf8_lossy(&bytes);
        let symbols = match self.extractor.extract(&absolute, &source_text) {
            Ok(symbols) => symbols,
            Err(error) => {
                warn!("extract failed for {}: {}", absolute.display(), error);
                Vec::new()
            }
        };

        Ok(FileIndex {
            path: source.path.clone(),
            language: source.language,
            hash,
            symbols,
        })
    }
}

fn file_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}
