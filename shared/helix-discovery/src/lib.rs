use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Not in a git repository")]
    NotInGitRepo,

    #[error("{marker} not found at git root: {searched}")]
    MarkerNotFound { marker: String, searched: PathBuf },

    #[error("Failed to get current directory: {0}")]
    CurrentDirError(#[from] std::io::Error),

    #[error("Failed to canonicalize path: {0}")]
    CanonicalizeError(String),
}

pub type Result<T> = std::result::Result<T, DiscoveryError>;

pub fn find_git_root(start: &Path) -> Result<PathBuf> {
    let mut current = start
        .canonicalize()
        .map_err(|e| DiscoveryError::CanonicalizeError(e.to_string()))?;

    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) if parent != current => current = parent.to_path_buf(),
            _ => return Err(DiscoveryError::NotInGitRepo),
        }
    }
}

pub fn find_git_root_from_cwd() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    find_git_root(&cwd)
}

pub fn find_marker(git_root: &Path, marker: &str) -> Result<PathBuf> {
    let marker_path = git_root.join(marker);

    if marker_path.exists() {
        Ok(marker_path)
    } else {
        Err(DiscoveryError::MarkerNotFound {
            marker: marker.to_string(),
            searched: git_root.to_path_buf(),
        })
    }
}

pub fn find_marker_from_cwd(marker: &str) -> Result<PathBuf> {
    let git_root = find_git_root_from_cwd()?;
    find_marker(&git_root, marker)
}

pub fn find_marker_or_create(git_root: &Path, marker: &str) -> Result<PathBuf> {
    let marker_path = git_root.join(marker);

    if !marker_path.exists() {
        std::fs::create_dir_all(&marker_path).map_err(|e| {
            DiscoveryError::CanonicalizeError(format!("Failed to create {marker}: {e}"))
        })?;
    }

    Ok(marker_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_git_repo() -> TempDir {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join(".git")).unwrap();
        temp
    }

    #[test]
    fn test_find_git_root_at_root() {
        let temp = setup_git_repo();
        let root = find_git_root(temp.path()).unwrap();
        assert_eq!(root, temp.path().canonicalize().unwrap());
    }

    #[test]
    fn test_find_git_root_from_subdir() {
        let temp = setup_git_repo();
        let subdir = temp.path().join("src").join("lib");
        fs::create_dir_all(&subdir).unwrap();

        let root = find_git_root(&subdir).unwrap();
        assert_eq!(root, temp.path().canonicalize().unwrap());
    }

    #[test]
    fn test_find_git_root_not_in_repo() {
        let temp = TempDir::new().unwrap();
        let result = find_git_root(temp.path());
        assert!(matches!(result, Err(DiscoveryError::NotInGitRepo)));
    }

    #[test]
    fn test_find_marker_exists() {
        let temp = setup_git_repo();
        fs::create_dir(temp.path().join(".decisions")).unwrap();

        let marker = find_marker(temp.path(), ".decisions").unwrap();
        assert_eq!(marker, temp.path().join(".decisions"));
    }

    #[test]
    fn test_find_marker_not_found() {
        let temp = setup_git_repo();

        let result = find_marker(temp.path(), ".decisions");
        assert!(matches!(result, Err(DiscoveryError::MarkerNotFound { .. })));
    }

    #[test]
    fn test_find_marker_or_create() {
        let temp = setup_git_repo();

        let marker = find_marker_or_create(temp.path(), ".decisions").unwrap();
        assert!(marker.exists());
        assert_eq!(marker, temp.path().join(".decisions"));
    }
}
