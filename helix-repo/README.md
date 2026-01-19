# helix-repo

Manage git repository clones for helix-tools. Clone repositories to a standard directory structure, organized by domain and path.

## Status

**Phase 0 - Project Foundation** (scaffolded)

The architecture and specs are defined. Implementation is pending.

## Quick Start (Planned)

```bash
# Clone a repository (multiple URL formats supported)
helix-repo clone https://github.com/facebook/react
helix-repo clone github.com/facebook/react
helix-repo clone git@github.com:facebook/react.git

# List cloned repositories
helix-repo list
helix-repo list --filter facebook

# Get repository info
helix-repo info facebook/react

# Remove a repository
helix-repo remove facebook/react

# Show the root directory
helix-repo root
```

## Why helix-repo?

Other helix-tools need local repository clones:

- **helix-docs** - Clone repos to extract documentation
- **helix-map** - Clone repos to index codebase structure
- **AI agents** - Clone repos to search OSS implementations

Extracting repo management to a shared tool:

- **Avoids duplication** - One clone per repo, shared across tools
- **Consistent layout** - `{root}/{domain}/{owner}/{repo}` everywhere
- **Single responsibility** - Each tool focuses on its core job

## Directory Structure

Repositories are cloned to `~/dev` by default:

```
~/dev/
├── github.com/
│   ├── facebook/
│   │   └── react/
│   └── vercel/
│       └── next.js/
├── gitlab.com/
│   └── inkscape/
│       └── inkscape/
└── git.sr.ht/
    └── ~sircmpwn/
        └── aerc/
```

## Features (Planned)

- **URL Flexibility** - HTTPS, SSH, and schemeless URLs
- **Global Cache** - Repos stored once, shared across tools
- **Library API** - Use from other Rust tools
- **Dry Run** - Preview operations without executing
- **JSON Output** - `--json` flag for AI agent consumption

## Configuration

### Global (`~/.helix/config/helix-repo.toml`)

```toml
[storage]
root = "~/dev"  # Default location
```

### Environment Variables

```bash
HELIX_REPO_ROOT=~/work/repos  # Override root directory
```

### CLI Flag

```bash
helix-repo clone --root ~/work/repos https://github.com/facebook/react
```

### Priority Order

1. `--root` CLI flag (highest)
2. `HELIX_REPO_ROOT` environment variable
3. Config file (`~/.helix/config/helix-repo.toml`)
4. Default: `~/dev` (lowest)

## Library Usage

helix-repo is both a CLI and a library crate:

```rust
use helix_repo::RepositoryManager;

// Initialize with default config
let manager = RepositoryManager::new()?;

// Clone a repo, returns path
let path = manager.clone("https://github.com/facebook/react")?;
// -> ~/dev/github.com/facebook/react

// Check if already cloned
if let Some(path) = manager.find("facebook/react")? {
    println!("Already cloned at: {}", path.display());
}

// List all repos
for repo in manager.list()? {
    println!("{}: {}", repo.name, repo.path.display());
}
```

## CLI Reference

| Command                    | Description                   |
| -------------------------- | ----------------------------- |
| `helix-repo clone <url>`   | Clone a repository            |
| `helix-repo list`          | List cloned repositories      |
| `helix-repo info <name>`   | Show repository details       |
| `helix-repo remove <name>` | Remove a cloned repository    |
| `helix-repo root`          | Print the root directory path |

### Clone Options

```bash
helix-repo clone <url> [OPTIONS]

Options:
  --root <path>      Override root directory
  --shallow          Shallow clone (depth 1)
  --branch <name>    Clone specific branch
  --dry-run          Print what would be done
  --json             Output as JSON
```

## Inspiration

- [ghq][ghq] - Multi-VCS repository management (Go)
- [git-grab][git-grab] - Simple git cloning to organized paths (Rust)

helix-repo takes git-grab's simplicity and adds library APIs for tool integration.

## Specifications

- [specs/requirements.md][requirements] - User stories and acceptance criteria
- [specs/design.md][design] - Architecture and data model
- [specs/tasks.md][tasks] - Implementation roadmap

## License

MIT

## Kiro Specs

- [Requirements][requirements]
- [Design][design]
- [Tasks][tasks]

<!-- Links -->

[requirements]: specs/requirements.md
[design]: specs/design.md
[tasks]: specs/tasks.md

<!-- Links -->

[requirements]: ./specs/requirements.md
[design]: ./specs/design.md
[tasks]: ./specs/tasks.md
[ghq]: https://github.com/x-motemen/ghq
[git-grab]: https://github.com/wezm/git-grab
