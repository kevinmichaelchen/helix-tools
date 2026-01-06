# helix-discovery

Project and git root discovery for helix-tools.

## Why

Multiple helix-tools need to find project markers:
- **helix-decisions** — Find `.decisions/` directory
- **hbd** — Find `.tickets/` directory
- **helix-config** — Find `.helix/` config directory

This crate provides unified discovery logic with consistent behavior.

## Usage

```rust
use helix_discovery::{find_git_root, find_marker, find_marker_from_cwd};

// Find git root from current directory
let git_root = find_git_root_from_cwd()?;

// Find git root from specific path
let git_root = find_git_root("/path/to/subdirectory")?;

// Find marker directory at git root
let decisions_dir = find_marker(&git_root, ".decisions")?;

// Convenience: find marker from cwd in one call
let decisions_dir = find_marker_from_cwd(".decisions")?;
```

## Discovery Algorithm

1. Start from the given directory (or cwd)
2. Walk up the directory tree looking for `.git/`
3. Once found, look for the marker at that level
4. Return error if marker not found at git root

## Error Handling

```rust
use helix_discovery::{DiscoveryError, find_marker_from_cwd};

match find_marker_from_cwd(".decisions") {
    Ok(path) => println!("Found: {}", path.display()),
    Err(DiscoveryError::NotInGitRepo) => {
        eprintln!("Error: Not in a git repository");
    }
    Err(DiscoveryError::MarkerNotFound { marker, searched }) => {
        eprintln!("Error: {} not found at {}", marker, searched.display());
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Consumers

| Crate | Marker |
|-------|--------|
| helix-decisions | `.decisions/` |
| hbd | `.tickets/` |
| helix-config | `.helix/` |

## License

MIT
