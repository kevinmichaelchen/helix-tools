# Ideas

Future tools and features for the helix-tools ecosystem.

---

## helix-recall

**Status:** Idea
**Priority:** High
**Complexity:** Medium

A developer activity reminder system — the "amnesia cure" for prolific developers
who touch many repos daily and lose track of context.

### Problem

When you're highly productive (65+ commits across 11+ repos in a week), context
gets lost:

- "What was I working on yesterday?"
- "Which repos did I touch this week?"
- "What PRs are still open?"

These questions require mental effort or manual git archaeology.

### Solution

A background daemon + CLI that:

1. **Polls local git repos** for recent commits, branches, uncommitted changes
2. **Fetches GitHub API** for PRs, reviews, issues, push events
3. **Stores everything in HelixDB** as a graph-vector cache
4. **Surfaces summaries** at shell startup or on-demand

### Why HelixDB?

| Capability | Use Case |
|------------|----------|
| **Graph storage** | Link commits → repos → PRs → issues |
| **Vector embeddings** | Semantic search: "what did I work on related to auth?" |
| **Embedded** | No server, works offline, fast reads |
| **`.db` file** | Single portable cache file |

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      HELIX-RECALL DAEMON                        │
│  (background service, polls on intervals)                       │
└─────────────────────────────────────────────────────────────────┘
         │                                    │
         ▼                                    ▼
┌─────────────────────┐           ┌─────────────────────┐
│   LOCAL GIT REPOS   │           │    GITHUB API       │
│  ~/dev/github.com/  │           │  gh api /events     │
│  kevinmichaelchen/  │           │  gh api /pulls      │
└─────────────────────┘           └─────────────────────┘
         │                                    │
         └────────────────┬───────────────────┘
                          ▼
              ┌───────────────────────┐
              │       HELIXDB         │
              │  ~/.cache/recall.db   │
              │                       │
              │  Nodes:               │
              │  • Commit (embedded)  │
              │  • Repo              │
              │  • PR                │
              │  • Issue             │
              │  • WorkSession       │
              │                       │
              │  Edges:               │
              │  • COMMITTED_TO      │
              │  • OPENED_PR         │
              │  • REVIEWED          │
              │  • TOUCHED_FILE      │
              └───────────────────────┘
                          │
                          ▼
              ┌───────────────────────┐
              │    SHELL / CLI        │
              │                       │
              │  recall              │
              │  recall yesterday    │
              │  recall --semantic   │
              │    "auth changes"    │
              └───────────────────────┘
```

### Data Model (HelixDB Graph)

```
Commit {
  sha: String,
  message: String,        // embedded for semantic search
  author: String,
  timestamp: DateTime,
  files_changed: Vec<String>,
}

Repo {
  name: String,
  path: String,
  remote_url: Option<String>,
}

PR {
  number: u32,
  title: String,          // embedded
  state: String,
  created_at: DateTime,
}

WorkSession {
  date: Date,
  repos_touched: Vec<String>,
  commit_count: u32,
  summary: String,        // AI-generated, embedded
}

// Edges
Commit -[COMMITTED_TO]-> Repo
Commit -[PART_OF]-> PR
PR -[BELONGS_TO]-> Repo
WorkSession -[INCLUDES]-> Commit
```

### CLI Interface

```bash
# Summaries
recall                      # Today's activity
recall yesterday            # Yesterday
recall week                 # Last 7 days
recall 2025-01-15           # Specific date

# Semantic search (vector similarity via HelixDB)
recall --semantic "authentication changes"
recall --semantic "bug fixes" --since=week

# Filtering
recall --repo=fire-dept
recall --repo=helix-tools --verbose

# Formats
recall --format=greeting    # Shell startup (compact)
recall --format=standup     # Daily standup text
recall --format=json        # Machine-readable

# Daemon control
recall daemon start
recall daemon stop
recall daemon status
recall sync                 # Force refresh now
```

### Shell Integration

```bash
# ~/.config/shell/recall.sh

if [[ $- == *i* ]] && command -v recall &> /dev/null; then
  recall --format=greeting --quiet
fi
```

Output on shell startup:

```
╭─ RECALL ──────────────────────────────────────────────────╮
│ TODAY: 3 repos, 8 commits                                 │
│ • helix-tools: "add recall daemon", "fix embeddings"      │
│ • dotfiles: "docs: AGENTS.md files"                       │
│ • fire-dept: "feat: scheduling API"                       │
│                                                           │
│ OPEN PRs: 2  │  WIP BRANCHES: 3                           │
╰───────────────────────────────────────────────────────────╯
```

### Daemon Behavior

| Source | Poll Interval | Trigger |
|--------|---------------|---------|
| Local git repos | 5 min | Also on `.git/index` change |
| GitHub events | 15 min | Rate-limit aware |
| PR/issue details | 1 hour | Or on event detection |

### Configuration

```toml
# ~/.config/recall/config.toml

[sources.local]
roots = ["~/dev/github.com/kevinmichaelchen"]
scan_depth = 1
author_patterns = ["kevin", "kevinmichaelchen"]

[sources.github]
username = "kevinmichaelchen"
include_private = true
include_orgs = ["my-company"]

[cache]
path = "~/.cache/recall.db"
embedding_model = "local"   # or "openai"

[display]
max_repos = 5
max_commits_per_repo = 3
```

### Implementation Notes

- Use `helix-graph-ops` for HelixDB interactions (shared crate)
- Reuse `helix-embeddings` for commit message embeddings
- Daemon: `tokio` runtime with interval tasks
- macOS: launchd plist for autostart
- Linux: systemd user service

### Related Tools

- **git-standup** — Inspiration for multi-repo scanning
- **gh-dash** — GitHub CLI dashboard patterns
- **hbd** — Similar HelixDB integration patterns to follow

### Open Questions

1. Should work sessions be auto-detected (gaps > 4 hours) or explicit?
2. AI summarization of daily activity — worth the latency/cost?
3. Team mode: show collaborator activity on shared repos?

---

## Future Ideas

### helix-timetrack

Automatic time tracking based on git activity and file watches. Infer time spent
per project without manual logging.

### helix-changelog

Auto-generate changelogs from commits + PR descriptions using semantic analysis.

### helix-deps

Dependency graph visualization and update tracking across repos.
