use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Storage not found at {path}")]
    NotFound { path: PathBuf },

    #[error("Failed to read storage: {source}")]
    ReadError {
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write storage: {source}")]
    WriteError {
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse storage: {source}")]
    ParseError {
        #[source]
        source: serde_json::Error,
    },

    #[error("Not in a git repository")]
    NotInGitRepo,

    #[error("Project marker not found: {marker}")]
    MarkerNotFound { marker: String },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug, Clone)]
pub enum StorageMode {
    ProjectLocal { tool_name: String },
    Global { tool_name: String },
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub mode: StorageMode,
    pub base_path: PathBuf,
}

impl StorageConfig {
    pub fn project_local(tool_name: &str) -> Result<Self> {
        let git_root = find_git_root(
            &std::env::current_dir().map_err(|e| StorageError::Other(e.to_string()))?,
        )?;
        let base_path = git_root.join(".helix").join("data").join(tool_name);

        Ok(Self {
            mode: StorageMode::ProjectLocal {
                tool_name: tool_name.to_string(),
            },
            base_path,
        })
    }

    pub fn global(tool_name: &str) -> Self {
        let base_path = helix_config::helix_data_dir().join(tool_name);

        Self {
            mode: StorageMode::Global {
                tool_name: tool_name.to_string(),
            },
            base_path,
        }
    }

    #[must_use]
    pub fn with_base_path(mut self, path: PathBuf) -> Self {
        self.base_path = path;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNode<T> {
    pub id: String,
    pub data: T,
    pub embedding: Option<Vec<f32>>,
    pub content_hash: String,
}

pub trait VectorStorage<T: Serialize + DeserializeOwned + Clone>: Send + Sync {
    fn insert(&mut self, node: StorageNode<T>) -> Result<()>;
    fn insert_batch(&mut self, nodes: Vec<StorageNode<T>>) -> Result<()>;
    fn get(&self, id: &str) -> Result<Option<StorageNode<T>>>;
    fn remove(&mut self, id: &str) -> Result<bool>;
    fn remove_batch(&mut self, ids: &[&str]) -> Result<usize>;
    fn search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<(StorageNode<T>, f32)>>;
    fn get_all_hashes(&self) -> Result<HashMap<String, String>>;
    fn list_ids(&self) -> Result<Vec<String>>;
    fn flush(&self) -> Result<()>;
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonStorageData<T> {
    nodes: HashMap<String, StorageNode<T>>,
}

impl<T> Default for JsonStorageData<T> {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }
}

pub struct JsonFileBackend<T: Serialize + DeserializeOwned + Clone> {
    config: StorageConfig,
    data: JsonStorageData<T>,
    dirty: bool,
}

impl<T: Serialize + DeserializeOwned + Clone> JsonFileBackend<T> {
    pub fn open(config: &StorageConfig) -> Result<Self> {
        let storage_file = config.base_path.join("index.json");

        let data = if storage_file.exists() {
            let content = fs::read_to_string(&storage_file)
                .map_err(|e| StorageError::ReadError { source: e })?;
            serde_json::from_str(&content).map_err(|e| StorageError::ParseError { source: e })?
        } else {
            JsonStorageData::default()
        };

        Ok(Self {
            config: config.clone(),
            data,
            dirty: false,
        })
    }

    fn save(&self) -> Result<()> {
        fs::create_dir_all(&self.config.base_path)
            .map_err(|e| StorageError::WriteError { source: e })?;

        let storage_file = self.config.base_path.join("index.json");
        let content = serde_json::to_string_pretty(&self.data)
            .map_err(|e| StorageError::ParseError { source: e })?;
        fs::write(&storage_file, content).map_err(|e| StorageError::WriteError { source: e })?;

        Ok(())
    }
}

impl<T: Serialize + DeserializeOwned + Clone + Send + Sync> VectorStorage<T>
    for JsonFileBackend<T>
{
    fn insert(&mut self, node: StorageNode<T>) -> Result<()> {
        self.data.nodes.insert(node.id.clone(), node);
        self.dirty = true;
        self.save()
    }

    fn insert_batch(&mut self, nodes: Vec<StorageNode<T>>) -> Result<()> {
        for node in nodes {
            self.data.nodes.insert(node.id.clone(), node);
        }
        self.dirty = true;
        self.save()
    }

    fn get(&self, id: &str) -> Result<Option<StorageNode<T>>> {
        Ok(self.data.nodes.get(id).cloned())
    }

    fn remove(&mut self, id: &str) -> Result<bool> {
        let removed = self.data.nodes.remove(id).is_some();
        if removed {
            self.dirty = true;
            self.save()?;
        }
        Ok(removed)
    }

    fn remove_batch(&mut self, ids: &[&str]) -> Result<usize> {
        let mut count = 0;
        for id in ids {
            if self.data.nodes.remove(*id).is_some() {
                count += 1;
            }
        }
        if count > 0 {
            self.dirty = true;
            self.save()?;
        }
        Ok(count)
    }

    fn search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<(StorageNode<T>, f32)>> {
        let mut results: Vec<(StorageNode<T>, f32)> = self
            .data
            .nodes
            .values()
            .filter_map(|node| {
                node.embedding.as_ref().map(|emb| {
                    let score = cosine_similarity(query_embedding, emb);
                    (node.clone(), score)
                })
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    fn get_all_hashes(&self) -> Result<HashMap<String, String>> {
        Ok(self
            .data
            .nodes
            .iter()
            .map(|(id, node)| (id.clone(), node.content_hash.clone()))
            .collect())
    }

    fn list_ids(&self) -> Result<Vec<String>> {
        Ok(self.data.nodes.keys().cloned().collect())
    }

    fn flush(&self) -> Result<()> {
        if self.dirty {
            self.save()?;
        }
        Ok(())
    }
}

impl<T: Serialize + DeserializeOwned + Clone> Drop for JsonFileBackend<T> {
    fn drop(&mut self) {
        if self.dirty {
            let _ = self.save();
        }
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

pub fn find_git_root(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) if parent != current => current = parent.to_path_buf(),
            _ => return Err(StorageError::NotInGitRepo),
        }
    }
}

pub fn project_hash(git_root: &Path) -> String {
    let path_str = git_root.to_string_lossy();
    let hash = blake3::hash(path_str.as_bytes());
    hex::encode(&hash.as_bytes()[..3])
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        title: String,
        value: i32,
    }

    fn setup_test_storage() -> (TempDir, JsonFileBackend<TestData>) {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            mode: StorageMode::ProjectLocal {
                tool_name: "test".to_string(),
            },
            base_path: temp_dir.path().to_path_buf(),
        };
        let storage = JsonFileBackend::open(&config).unwrap();
        (temp_dir, storage)
    }

    #[test]
    fn test_insert_and_get() {
        let (_temp, mut storage) = setup_test_storage();

        let node = StorageNode {
            id: "test-1".to_string(),
            data: TestData {
                title: "Test".to_string(),
                value: 42,
            },
            embedding: Some(vec![1.0, 0.0, 0.0]),
            content_hash: "hash123".to_string(),
        };

        storage.insert(node).unwrap();

        let retrieved = storage.get("test-1").unwrap().unwrap();
        assert_eq!(retrieved.id, "test-1");
        assert_eq!(retrieved.data.title, "Test");
        assert_eq!(retrieved.data.value, 42);
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            mode: StorageMode::ProjectLocal {
                tool_name: "test".to_string(),
            },
            base_path: temp_dir.path().to_path_buf(),
        };

        {
            let mut storage: JsonFileBackend<TestData> = JsonFileBackend::open(&config).unwrap();
            storage
                .insert(StorageNode {
                    id: "persist-1".to_string(),
                    data: TestData {
                        title: "Persisted".to_string(),
                        value: 100,
                    },
                    embedding: None,
                    content_hash: "hash".to_string(),
                })
                .unwrap();
        }

        let storage: JsonFileBackend<TestData> = JsonFileBackend::open(&config).unwrap();
        let retrieved = storage.get("persist-1").unwrap().unwrap();
        assert_eq!(retrieved.data.title, "Persisted");
    }

    #[test]
    fn test_search() {
        let (_temp, mut storage) = setup_test_storage();

        storage
            .insert(StorageNode {
                id: "vec-1".to_string(),
                data: TestData {
                    title: "A".to_string(),
                    value: 1,
                },
                embedding: Some(vec![1.0, 0.0, 0.0]),
                content_hash: "h1".to_string(),
            })
            .unwrap();

        storage
            .insert(StorageNode {
                id: "vec-2".to_string(),
                data: TestData {
                    title: "B".to_string(),
                    value: 2,
                },
                embedding: Some(vec![0.0, 1.0, 0.0]),
                content_hash: "h2".to_string(),
            })
            .unwrap();

        let results = storage.search(&[1.0, 0.0, 0.0], 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0.id, "vec-1");
        assert!((results[0].1 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_remove() {
        let (_temp, mut storage) = setup_test_storage();

        storage
            .insert(StorageNode {
                id: "remove-1".to_string(),
                data: TestData {
                    title: "ToRemove".to_string(),
                    value: 1,
                },
                embedding: None,
                content_hash: "h".to_string(),
            })
            .unwrap();

        assert!(storage.get("remove-1").unwrap().is_some());
        assert!(storage.remove("remove-1").unwrap());
        assert!(storage.get("remove-1").unwrap().is_none());
    }

    #[test]
    fn test_get_all_hashes() {
        let (_temp, mut storage) = setup_test_storage();

        storage
            .insert(StorageNode {
                id: "h1".to_string(),
                data: TestData {
                    title: "A".to_string(),
                    value: 1,
                },
                embedding: None,
                content_hash: "hash-a".to_string(),
            })
            .unwrap();

        storage
            .insert(StorageNode {
                id: "h2".to_string(),
                data: TestData {
                    title: "B".to_string(),
                    value: 2,
                },
                embedding: None,
                content_hash: "hash-b".to_string(),
            })
            .unwrap();

        let hashes = storage.get_all_hashes().unwrap();
        assert_eq!(hashes.len(), 2);
        assert_eq!(hashes.get("h1"), Some(&"hash-a".to_string()));
        assert_eq!(hashes.get("h2"), Some(&"hash-b".to_string()));
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 0.001);

        let d = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &d) + 1.0).abs() < 0.001);
    }
}
