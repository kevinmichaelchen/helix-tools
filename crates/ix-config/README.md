# ix-config

Hierarchical configuration loading for Ixchel. Provides a consistent config
loading pattern across CLI tools in the ecosystem.

## Why

Ixchel tools share common configuration patterns:

- Global config in `~/.ixchel/config/`
- Project config in `.ixchel/`
- Environment variable overrides
- Shared settings (GitHub token, embedding model, etc.)

This crate centralizes that logic to avoid divergence.

## Directory Structure

Ixchel uses a unified `~/.ixchel/` directory for global config/state/data:

```
~/.ixchel/                        # Unified ixchel home
├── config/                       # Configuration (user-editable TOML)
│   ├── config.toml               # Shared settings (GitHub token, embedding model)
│   └── ixchel.toml               # ixchel settings
│
├── data/                         # Caches & databases (auto-generated)
├── state/                        # Runtime metadata (ephemeral)
└── log/                          # Operation logs
```

Project-local overrides:

```
{project}/.ixchel/                # Project config + canonical artifacts
├── config.toml                   # Project shared config
├── ixchel.toml                   # ixchel project overrides
└── ...
```

See the [Configuration docs][config-docs] for full details.

## Configuration Hierarchy

```
Priority (highest to lowest):
1. Environment variables (IXCHEL_*, GITHUB_TOKEN, etc.)
2. Global tool config (~/.ixchel/config/<tool>.toml)
3. Global shared config (~/.ixchel/config/config.toml)
4. Project tool config (.ixchel/<tool>.toml)
5. Project shared config (.ixchel/config.toml)
6. Defaults (from Default trait)
```

## Usage

```rust
use ix_config::{ConfigLoader, load_config};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
struct MyToolConfig {
    #[serde(default)]
    concurrency: usize,
}

// Simple loading with defaults
let config: MyToolConfig = load_config("my-tool")?;

// Or with more control
let loader = ConfigLoader::new("my-tool");
let config: MyToolConfig = loader
    .with_env_prefix("MY_TOOL")
    .load()?;
```

## Shared Config

The global `~/.ixchel/config/config.toml` contains settings shared across tools:

```toml
[github]
token = "ghp_xxx"  # Or use GITHUB_TOKEN env var

[embedding]
model = "BAAI/bge-small-en-v1.5"
batch_size = 32

[storage]
backend = "helixdb"
path = "data/ixchel" # relative to .ixchel/
```

## Path Helpers

```rust
use ix_config::{ixchel_home, ixchel_config_dir, ixchel_data_dir};

// Get the Ixchel home directory
let home = ixchel_home();  // ~/.ixchel

// Get specific subdirectories
let config_dir = ixchel_config_dir();  // ~/.ixchel/config
let data_dir = ixchel_data_dir();      // ~/.ixchel/data

// Tool-specific data directories
let index_dir = ixchel_data_dir().join("index");  // ~/.ixchel/data/index
```

## GitHub Token Detection

The crate provides automatic GitHub token detection:

```rust
use ix_config::detect_github_token;

if let Some(token) = detect_github_token() {
    // Token found from: GITHUB_TOKEN, GH_TOKEN, config, or `gh auth token`
}
```

Detection order:

1. `GITHUB_TOKEN` environment variable
2. `GH_TOKEN` environment variable
3. `github.token` in config files
4. `gh auth token` command output

## Environment Variables

```bash
# Override ixchel home directory
export IXCHEL_HOME=~/my-ixchel

# Shared settings
export IXCHEL_GITHUB_TOKEN=ghp_xxx
export IXCHEL_EMBEDDING_MODEL=jina-embeddings-v3

# Tool-specific overrides (using __ for nesting)
export IXCHEL__SYNC__CONCURRENCY=10
```

## Consumers (current)

| Crate           | Notes                                |
| --------------- | ------------------------------------ |
| `ix-core`       | Loads merged Ixchel config           |
| `ix-embeddings` | Reads embedding settings from config |
| `ix-app`        | Selects backends for apps            |

## Specifications

- [specs/requirements.md][requirements] - Requirements in EARS notation
- [specs/design.md][design] - Design decisions and API details
- [specs/tasks.md][tasks] - Implementation plan and backlog

<!-- Links -->

[config-docs]: https://kevinmichaelchen.github.io/ixchel/docs/configuration
[requirements]: ./specs/requirements.md
[design]: ./specs/design.md
[tasks]: ./specs/tasks.md
