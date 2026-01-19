# HBD AGENTS

**Parent:** See `../AGENTS.md` for workspace context.

## Overview

Git-first issue tracker. Issues are stored as Markdown with YAML frontmatter in
`.ixchel/issues/` and manipulated via the `hbd` CLI.

## Structure

```
hbd/
├── src/
│   ├── main.rs            # CLI entrypoint
│   ├── lib.rs             # Library exports
│   ├── commands/          # CLI subcommand handlers
│   ├── storage.rs         # TicketStore file I/O
│   ├── markdown.rs        # Frontmatter serialize/parse
│   ├── types.rs           # Issue/Status/Priority/etc.
│   ├── domain/            # Graph + filters
│   ├── id.rs              # ID generation helpers
│   └── db.rs              # HelixDB placeholders (todo)
├── specs/                 # requirements/design/tasks
└── README.md              # CLI reference + format
```

## Where To Look

| Task                   | Location                                   |
| ---------------------- | ------------------------------------------ |
| Add a CLI command      | `hbd/src/main.rs` + `hbd/src/commands/`    |
| Modify issue schema    | `hbd/src/types.rs` + `hbd/src/markdown.rs` |
| Ticket file I/O        | `hbd/src/storage.rs`                       |
| Dependency graph logic | `hbd/src/domain/graph.rs`                  |
| ID format              | `hbd/src/id.rs`                            |
| HelixDB integration    | `hbd/src/db.rs` (currently todo)           |

## Commands

```bash
cargo run -p hbd -- --help
cargo run -p hbd -- init
cargo run -p hbd -- create "Title" --type bug
```
