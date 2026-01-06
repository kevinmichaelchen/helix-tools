# helix-discovery: Design Specification

**Document:** design.md  
**Status:** Active (2026-01-06)  
**Author:** Kevin Chen

## Overview

helix-discovery provides project and git root discovery for helix-tools. It locates project markers (like `.decisions/`, `.tickets/`, `.helix/`) by walking up the directory tree from the current working directory.

## Design Goals

1. **Simplicity** — Single algorithm for all marker discovery
2. **Git-anchored** — Markers must be at git root (not arbitrary ancestors)
3. **Consistent** — All tools use the same discovery logic
4. **Fast** — Minimal filesystem operations

## Algorithm

```
find_marker(start_dir, marker_name):
    1. Walk up from start_dir looking for .git/
    2. If .git/ not found → NotInGitRepo error
    3. At git root, check if marker exists
    4. If marker exists → return path
    5. If marker missing → MarkerNotFound error
```

### Why Git-Anchored?

Markers at git root ensures:
- Predictable location (always at repo root)
- Works with monorepos (each repo has its own markers)
- Avoids ambiguity with nested projects
- Matches user mental model ("this repo's decisions")

## API Design

### Core Functions

```rust
/// Find git root from current directory
pub fn find_git_root_from_cwd() -> Result<PathBuf, DiscoveryError>;

/// Find git root from specific path
pub fn find_git_root(start: impl AsRef<Path>) -> Result<PathBuf, DiscoveryError>;

/// Find marker directory at git root
pub fn find_marker(git_root: &Path, marker: &str) -> Result<PathBuf, DiscoveryError>;

/// Convenience: find marker from cwd in one call
pub fn find_marker_from_cwd(marker: &str) -> Result<PathBuf, DiscoveryError>;
```

### Error Types

```rust
#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Not in a git repository")]
    NotInGitRepo,
    
    #[error("{marker} not found at {}", searched.display())]
    MarkerNotFound {
        marker: String,
        searched: PathBuf,
    },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Usage Examples

### helix-decisions

```rust
use helix_discovery::find_marker_from_cwd;

fn find_decisions_dir() -> Result<PathBuf, DiscoveryError> {
    find_marker_from_cwd(".decisions")
}
```

### hbd

```rust
use helix_discovery::find_marker_from_cwd;

fn find_tickets_dir() -> Result<PathBuf, DiscoveryError> {
    find_marker_from_cwd(".tickets")
}
```

### helix-config

```rust
use helix_discovery::{find_git_root_from_cwd, find_marker};

fn find_project_config() -> Option<PathBuf> {
    let git_root = find_git_root_from_cwd().ok()?;
    find_marker(&git_root, ".helix").ok()
}
```

## Implementation

### Git Root Detection

```rust
pub fn find_git_root(start: impl AsRef<Path>) -> Result<PathBuf, DiscoveryError> {
    let mut current = start.as_ref().canonicalize()?;
    
    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }
        
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return Err(DiscoveryError::NotInGitRepo),
        }
    }
}
```

### Marker Discovery

```rust
pub fn find_marker(git_root: &Path, marker: &str) -> Result<PathBuf, DiscoveryError> {
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
```

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| Not in git repo | `NotInGitRepo` error |
| Marker doesn't exist | `MarkerNotFound` error with searched path |
| Bare git repo | Works (`.git` is directory, not file) |
| Git worktree | Works (`.git` file points to main repo) |
| Symlinked marker | Follows symlink |
| Read permission denied | `Io` error |

## Performance

| Operation | Expected |
|-----------|----------|
| find_git_root | < 1ms (typically 1-5 stat calls) |
| find_marker | < 1ms (1 stat call) |

## Consumers

| Tool | Marker |
|------|--------|
| helix-decisions | `.decisions/` |
| hbd | `.tickets/` |
| helix-config | `.helix/` |

## See Also

- [requirements.md](./requirements.md) — Requirements specification
- [helix-config/specs/design.md](../helix-config/specs/design.md) — Uses discovery for project config
