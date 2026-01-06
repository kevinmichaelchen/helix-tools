# helix-discovery: Requirements Specification

**Document:** requirements.md  
**Status:** Active (2026-01-06)  
**Author:** Kevin Chen

## Vision

helix-discovery provides git root and project marker discovery for helix-tools, ensuring consistent behavior across all tools when locating project-specific directories.

## User Stories

### For Tool Developers
```
As a helix-tools developer,
I want consistent marker discovery logic,
So that all tools find project directories the same way.
```

**Acceptance Criteria:**
- Single `find_marker_from_cwd()` function
- Returns absolute path to marker directory
- Clear errors for "not in git repo" vs "marker not found"

### For End Users
```
As a developer using helix-tools,
I want tools to auto-discover project directories,
So that I don't have to specify paths manually.
```

**Acceptance Criteria:**
- `helix-decisions search` finds `.decisions/` automatically
- `hbd list` finds `.tickets/` automatically
- Works from any subdirectory within the repo

## Functional Requirements

### FR-1: Git Root Discovery
- **EARS:** The system SHALL find the git repository root by walking up from a given path.
- **Input:** Starting directory path (or cwd)
- **Output:** Absolute path to git root
- **Error:** `NotInGitRepo` if no `.git/` found

### FR-2: Marker Discovery
- **EARS:** The system SHALL find a named marker directory at the git root.
- **Input:** Git root path + marker name (e.g., ".decisions")
- **Output:** Absolute path to marker directory
- **Error:** `MarkerNotFound` if marker doesn't exist at git root

### FR-3: Combined Discovery
- **EARS:** The system SHALL provide a convenience function combining FR-1 and FR-2.
- **Input:** Marker name
- **Output:** Absolute path to marker directory
- **Behavior:** Find git root from cwd, then find marker at that root

## Non-Functional Requirements

### Performance
- Git root discovery: < 1ms
- Marker check: < 1ms

### Reliability
- Handle symlinks correctly
- Handle permission errors gracefully
- Work on all platforms (Linux, macOS, Windows)

## Markers by Tool

| Tool | Marker | Purpose |
|------|--------|---------|
| helix-decisions | `.decisions/` | Decision documents |
| hbd | `.tickets/` | Issue tracking |
| helix-config | `.helix/` | Project configuration |

## Out of Scope

- Creating markers (tools create their own)
- Validating marker contents
- Multi-repo discovery (one repo at a time)

## See Also

- [design.md](./design.md) — Architecture and API
