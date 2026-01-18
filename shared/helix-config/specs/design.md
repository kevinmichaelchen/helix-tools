# Design

**Document:** design.md  
**Status:** Active (2026-01-06)  
**Author:** Kevin Chen

This document describes the design decisions and implementation details for `helix-config`.

## Overview

`helix-config` provides hierarchical configuration loading for helix-tools. It implements a unified directory structure with layered config from global, project, and environment sources.

## Design Goals

1. **Unified Home** - Single `~/.helix/` directory for all helix data
2. **Consistency** - All helix tools use the same config loading pattern
3. **Predictability** - Clear precedence rules for config merging
4. **Convenience** - Auto-detection of common settings (GitHub token)
5. **Simplicity** - Minimal API surface, easy to use correctly

## Directory Structure

### Unified ~/.helix/ Home

```
~/.helix/                         # Helix root directory
├── config/                       # Configuration files (user-editable)
│   ├── config.toml               # Shared settings
│   ├── hbd.toml                  # hbd-specific
│   ├── helix-docs.toml           # helix-docs-specific
│   ├── helix-map.toml            # helix-map-specific
│   ├── helix-repo.toml           # helix-repo-specific
│   └── helix-mail.toml           # helix-mail-specific
│
├── data/                         # Caches & databases (auto-generated)
│   ├── docs/                     # helix-docs cache (HelixDB)
│   ├── repos/                    # helix-repo clones
│   └── index/                    # helix-map index (HelixDB)
│
├── state/                        # Runtime metadata (ephemeral)
│   ├── agents/                   # Agent registry
│   ├── locks/                    # Process locks
│   └── cache/                    # Computed caches
│
├── log/                          # Operation logs
│   ├── helix-docs.log
│   ├── helix-map.log
│   └── helix-repo.log
│
└── .helix-version                # Metadata
```

### Project Local

```
{project}/.helix/                 # Project config only (no data)
├── config.toml                   # Project shared config
├── hbd.toml                      # hbd project overrides
├── helix-docs.toml               # helix-docs project overrides
└── ...
```

### Why Unified?

**Old model (problematic):**
```
~/.config/helix/     ← configs
~/.cache/helix/      ← data
```

**New model (unified):**
```
~/.helix/            ← everything
├── config/          ← configs (user edits)
├── data/            ← caches (auto-generated)
├── state/           ← metadata (ephemeral)
└── log/             ← debugging
```

Benefits:
- Single location (easy to manage, backup, reset)
- Clear separation (config vs data vs state)
- Follows Rust/Node/Ruby tool patterns
- Clear what's safe to delete

---

## Configuration Hierarchy

```
┌─────────────────────────────────────────────────────────┐
│                  Environment Variables                   │  Priority 1 (highest)
│           HELIX_HOME, HELIX_GITHUB__TOKEN, etc.          │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                  Project Tool Config                     │  Priority 2
│                  .helix/<tool>.toml                      │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                 Project Shared Config                    │  Priority 3
│                   .helix/config.toml                     │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                  Global Tool Config                      │  Priority 4
│              ~/.helix/config/<tool>.toml                 │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                 Global Shared Config                     │  Priority 5
│               ~/.helix/config/config.toml                │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                      Defaults                            │  Priority 6 (lowest)
│                  (from Default trait)                    │
└─────────────────────────────────────────────────────────┘
```

---

## Shared Config Schema

The shared `config.toml` contains settings used by multiple tools:

```toml
[github]
token = "ghp_xxx"  # Most tools need GitHub access

[embedding]
provider = "fastembed"
model = "BAAI/bge-small-en-v1.5"  # helix-docs, helix-map
batch_size = 32
dimension = 384

[storage]
base = "~/.helix/data"  # Override data location if needed

[agent]
registry_url = "https://..."  # For helix-mail
```

### SharedConfig Struct

```rust
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SharedConfig {
    #[serde(default)]
    pub github: GitHubConfig,
    #[serde(default)]
    pub embedding: EmbeddingConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GitHubConfig {
    pub token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingConfig {
    #[serde(default = "default_embedding_provider")]
    pub provider: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default)]
    pub dimension: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_storage_base")]
    pub base: PathBuf,
}
```

---

## API Design

### Path Helpers

```rust
/// Get the helix home directory (~/.helix or $HELIX_HOME)
pub fn helix_home() -> PathBuf {
    if let Ok(home) = std::env::var("HELIX_HOME") {
        return PathBuf::from(home);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".helix")
}

/// Get the config directory (~/.helix/config)
pub fn helix_config_dir() -> PathBuf {
    helix_home().join("config")
}

/// Get the data directory (~/.helix/data)
pub fn helix_data_dir() -> PathBuf {
    helix_home().join("data")
}

/// Get the state directory (~/.helix/state)
pub fn helix_state_dir() -> PathBuf {
    helix_home().join("state")
}

/// Get the log directory (~/.helix/log)
pub fn helix_log_dir() -> PathBuf {
    helix_home().join("log")
}

/// Find project config directory (.helix/) by walking up from cwd
pub fn find_project_config_dir() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let helix_dir = cwd.join(".helix");
    helix_dir.exists().then_some(helix_dir)
}
```

### Config Loading

```rust
/// Simple config loading with defaults
pub fn load_config<T: DeserializeOwned + Default>(tool_name: &str) -> Result<T, ConfigError> {
    ConfigLoader::new(tool_name).load()
}

/// Load shared config (github token, embedding model, etc.)
pub fn load_shared_config() -> Result<SharedConfig, ConfigError> {
    ConfigLoader::new("").load_shared()
}
```

### Builder API

```rust
pub struct ConfigLoader {
    tool_name: String,
    env_prefix: Option<String>,
    project_dir: Option<PathBuf>,
    global_dir: Option<PathBuf>,
}

impl ConfigLoader {
    pub fn new(tool_name: impl Into<String>) -> Self;
    
    /// Set custom environment variable prefix (default: HELIX_{TOOL})
    pub fn with_env_prefix(self, prefix: impl Into<String>) -> Self;
    
    /// Override project config directory
    pub fn with_project_dir(self, path: impl Into<PathBuf>) -> Self;
    
    /// Override global config directory
    pub fn with_global_dir(self, path: impl Into<PathBuf>) -> Self;
    
    /// Load and merge config from all sources
    pub fn load<T: DeserializeOwned + Default>(self) -> Result<T, ConfigError>;
    
    /// Load shared config only
    pub fn load_shared(self) -> Result<SharedConfig, ConfigError>;
}
```

### GitHub Token Helper

```rust
/// Detect GitHub token from multiple sources
pub fn detect_github_token() -> Option<String> {
    // 1. Environment variables (highest priority)
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        return Some(token);
    }
    if let Ok(token) = std::env::var("GH_TOKEN") {
        return Some(token);
    }
    
    // 2. Config file
    if let Ok(config) = load_shared_config() {
        if let Some(token) = config.github.token {
            return Some(token);
        }
    }
    
    // 3. gh CLI fallback
    std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
```

---

## Merge Strategy

### Scalar Values
Later values replace earlier values.

### Tables (Objects)
Merged recursively. Keys in higher-priority configs override lower-priority.

### Arrays
Replaced entirely (not concatenated). This prevents unexpected behavior.

### Example

```toml
# ~/.helix/config/config.toml (global)
[github]
token = "global_token"

[embedding]
provider = "fastembed"
model = "small-model"
batch_size = 16
dimension = 384

# .helix/config.toml (project)
[embedding]
batch_size = 32  # Override just this field

# Result:
# github.token = "global_token"
# embedding.provider = "fastembed"
# embedding.model = "small-model"
# embedding.batch_size = 32
# embedding.dimension = 384
```

---

## Error Types

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file {}: {source}", path.display())]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config file {}: {source}", path.display())]
    ParseError {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Invalid config value: {message}")]
    ValidationError {
        message: String,
    },
    
    #[error("Home directory not found")]
    HomeDirNotFound,
}
```

---

## Security Considerations

### Token Handling
- Never log token values
- Never include tokens in error messages
- Consider `secrecy` crate for memory safety

### File Permissions
- Warn if config files are world-readable
- Recommend `chmod 600` for files containing secrets

---

## Implementation Notes

### Loading Order

```rust
pub fn load<T: DeserializeOwned + Default>(self) -> Result<T, ConfigError> {
    let mut merged = toml::Table::new();

    // 1. Global shared config: ~/.helix/config/config.toml
    let global_dir = self.global_dir.unwrap_or_else(helix_config_dir);
    if let Some(table) = load_toml_file(&global_dir.join("config.toml"))? {
        merge_tables(&mut merged, table);
    }

    // 2. Global tool config: ~/.helix/config/<tool>.toml
    let tool_config = global_dir.join(format!("{}.toml", self.tool_name));
    if let Some(table) = load_toml_file(&tool_config)? {
        merge_tables(&mut merged, table);
    }

    // 3. Project shared config: .helix/config.toml
    let project_dir = self.project_dir.or_else(find_project_config_dir);
    if let Some(dir) = &project_dir {
        if let Some(table) = load_toml_file(&dir.join("config.toml"))? {
            merge_tables(&mut merged, table);
        }
    }

    // 4. Project tool config: .helix/<tool>.toml
    if let Some(dir) = project_dir {
        let tool_config = dir.join(format!("{}.toml", self.tool_name));
        if let Some(table) = load_toml_file(&tool_config)? {
            merge_tables(&mut merged, table);
        }
    }

    // 5. Environment variables (TODO: implement env overlay)

    // 6. Default if empty
    if merged.is_empty() {
        return Ok(T::default());
    }

    // Deserialize merged config
    let value = toml::Value::Table(merged);
    value.try_into().map_err(|e| ConfigError::ParseError {
        path: PathBuf::from("<merged>"),
        source: e,
    })
}
```

---

## Consumers

| Crate | Tool Name | Config Path | Data Path |
|-------|-----------|-------------|-----------|
| `hbd` | `hbd` | `~/.helix/config/hbd.toml` | `.tickets/` (project) |
| `helix-docs` | `helix-docs` | `~/.helix/config/helix-docs.toml` | `~/.helix/data/docs/` |
| `helix-map` | `helix-map` | `~/.helix/config/helix-map.toml` | `~/.helix/data/index/` |
| `helix-repo` | `helix-repo` | `~/.helix/config/helix-repo.toml` | `~/.helix/data/repos/` |
| `helix-mail` | `helix-mail` | `~/.helix/config/helix-mail.toml` | `~/.helix/state/agents/` |

---

## Migration Guide

### From Old Directory Structure

Old:
```
~/.config/helix/config.toml
~/.config/helix/helix-docs.toml
~/.cache/helix/docs/
```

New:
```
~/.helix/config/config.toml
~/.helix/config/helix-docs.toml
~/.helix/data/docs/
```

Migration steps:
1. Create `~/.helix/` directory
2. Move `~/.config/helix/*` to `~/.helix/config/`
3. Move `~/.cache/helix/*` to `~/.helix/data/`
4. Remove old directories

### For Tool Developers

1. Replace custom config loading with `helix_config::load_config("tool-name")`
2. Use `helix_config::helix_data_dir().join("tool-data")` for data storage
3. Use `helix_config::detect_github_token()` for GitHub access

---

## Storage Path Conventions

> **Note:** helix-config does NOT depend on HelixDB. It only documents path conventions that tools should follow when they use HelixDB as their storage backend.
>
> See [ADR-004](../../.decisions/004-trait-based-storage-architecture.md) for why storage is each tool's concern.

### Project-Local Storage (Default)

For tools that store data within the project:

```
{project}/.helix/data/{tool}/
├── data.mdb                 # LMDB data file (if using HelixDB)
└── lock.mdb                 # LMDB lock file
```

**Example:** helix-decisions stores decision graph + vectors in `.helix/data/decisions/`

### Global Storage

For tools with cross-project data:

```
~/.helix/data/{tool}/
├── data.mdb
└── lock.mdb
```

**Example:** helix-docs stores documentation cache in `~/.helix/data/docs/`

### Path Helper Functions

```rust
use helix_config::{helix_data_dir, project_data_dir};

// Global data path
let global_data = helix_data_dir().join("docs");
// → ~/.helix/data/docs/

// Project-local data path (requires git root discovery)
let project_data = project_data_dir("decisions")?;
// → {project}/.helix/data/decisions/
```

### Storage Config Options

Tools can configure storage behavior via shared config:

```toml
# ~/.helix/config/config.toml

[storage]
# LMDB map size (default: 1GB for project-local, 10GB for global)
map_size_mb = 1024

# Max readers (default: 126)
max_readers = 126
```

Tools read this config and pass it to their storage backend implementation.

---

## See Also

- [requirements.md](./requirements.md) — Requirements specification
- [ADR-004](../../.decisions/004-trait-based-storage-architecture.md) — Trait-based storage architecture
- [shared/AGENTS.md](../AGENTS.md) — Shared crates overview
