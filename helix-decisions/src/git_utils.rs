//! Git utilities for listing tracked decision files.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub enum ListError {
    GitNotAvailable,
    NotAGitRepo,
    CommandFailed(String),
    DirectoryNotFound,
}

impl std::fmt::Display for ListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitNotAvailable => write!(f, "git command not available"),
            Self::NotAGitRepo => write!(f, "not a git repository"),
            Self::CommandFailed(msg) => write!(f, "git command failed: {msg}"),
            Self::DirectoryNotFound => write!(f, "directory not found"),
        }
    }
}

impl std::error::Error for ListError {}

pub fn list_decision_files(
    repo_root: &Path,
    decisions_dir: &Path,
) -> Result<Vec<PathBuf>, ListError> {
    match git_ls_files(repo_root, decisions_dir) {
        Ok(files) => Ok(files),
        Err(_) => walk_directory(decisions_dir),
    }
}

fn git_ls_files(repo_root: &Path, decisions_dir: &Path) -> Result<Vec<PathBuf>, ListError> {
    let relative_dir = decisions_dir
        .strip_prefix(repo_root)
        .unwrap_or(decisions_dir);

    let pattern = format!("{}/**/*.md", relative_dir.display());

    let output = Command::new("git")
        .args(["ls-files", "--", &pattern])
        .current_dir(repo_root)
        .output()
        .map_err(|_| ListError::GitNotAvailable)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(ListError::NotAGitRepo);
        }
        return Err(ListError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files: Vec<PathBuf> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| repo_root.join(line))
        .collect();

    files.sort();
    Ok(files)
}

fn walk_directory(dir: &Path) -> Result<Vec<PathBuf>, ListError> {
    if !dir.exists() {
        return Err(ListError::DirectoryNotFound);
    }

    let mut files = Vec::new();
    walk_recursive(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), ListError> {
    let entries = std::fs::read_dir(dir).map_err(|_| ListError::DirectoryNotFound)?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            walk_recursive(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_structure(temp: &TempDir) -> PathBuf {
        let decisions = temp.path().join(".decisions");
        fs::create_dir_all(&decisions).unwrap();

        fs::write(decisions.join("001-first.md"), "# First").unwrap();
        fs::write(decisions.join("002-second.md"), "# Second").unwrap();

        let subdir = decisions.join("archive");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("003-archived.md"), "# Archived").unwrap();

        fs::write(decisions.join("notes.txt"), "Not markdown").unwrap();

        decisions
    }

    #[test]
    fn test_walk_directory_finds_md_files() {
        let temp = TempDir::new().unwrap();
        let decisions = create_test_structure(&temp);

        let files = walk_directory(&decisions).unwrap();

        assert_eq!(files.len(), 3);

        let filenames: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(filenames.contains(&"001-first.md".to_string()));
        assert!(filenames.contains(&"002-second.md".to_string()));
        assert!(filenames.contains(&"003-archived.md".to_string()));
        assert!(!filenames.contains(&"notes.txt".to_string()));
    }

    #[test]
    fn test_walk_directory_sorted() {
        let temp = TempDir::new().unwrap();
        let decisions = create_test_structure(&temp);

        let files = walk_directory(&decisions).unwrap();

        for i in 1..files.len() {
            assert!(files[i - 1] < files[i], "files should be sorted");
        }
    }

    #[test]
    fn test_walk_directory_not_found() {
        let result = walk_directory(Path::new("/nonexistent/path"));
        assert!(matches!(result, Err(ListError::DirectoryNotFound)));
    }

    #[test]
    fn test_list_decision_files_fallback() {
        let temp = TempDir::new().unwrap();
        let decisions = create_test_structure(&temp);

        let files = list_decision_files(temp.path(), &decisions).unwrap();
        assert_eq!(files.len(), 3);
    }
}
