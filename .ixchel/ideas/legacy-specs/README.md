# Ixchel (Blueprints)

Note: `./ixchel` is a brainstorming/specs directory. Many docs here started under
the working name “Ixchel”; the implemented tool is now **Ixchel** (`ix-cli/`,
`ix-core/`, `ix-mcp/`, `ix-storage-helixdb/`) and the canonical on-disk directory
is `.ixchel/`.

Mapping (when you see older terminology in these docs):

- `ixchel` CLI → `ixchel`
- `.ixchel/` dir → `.ixchel/`
- HelixDB stays HelixDB

Ixchel is a git-first knowledge weaving system: Markdown is canonical; HelixDB
is a rebuildable local cache for graph/vector queries.

## Quick Start (implemented)

From the workspace root:

```bash
cargo install --path ix-cli
cargo install --path ix-mcp # optional: MCP server binary (ixchel-mcp)
```

In a git repo:

```bash
ixchel init
ixchel create decision "Use PostgreSQL for primary storage" --status proposed
ixchel create issue "Implement connection pooling" --status open

# Link entities via frontmatter relationships
ixchel link iss-xxxx implements dec-xxxx

# Build/rebuild the local HelixDB cache, then search
ixchel sync
ixchel search "database performance"
```

Edit an entity in your `$EDITOR`:

```bash
ixchel edit dec-xxxx
```

## Storage Model

- Source of truth (git-tracked): `.ixchel/**/*.md`
- Rebuildable cache (gitignored): `.ixchel/data/` (defaults to `.ixchel/data/ixchel/`)
- Embedding models cache (gitignored): `.ixchel/models/`

## Documentation (blueprints)

- [Vision](./specs/vision.md)
- [Entities](./specs/entities.md)
- [Architecture](./specs/architecture.md)
- [Graph Schema](./specs/graph-schema.md)
- [Workspace Blueprint](./specs/workspace-blueprint.md)
- [Research: chunking + link inference](./specs/research/chunking-and-link-inference.md)
- [CLI spec (draft)](./specs/cli.md)
- [Extensibility spec (draft)](./specs/extensibility.md)
- [TUI spec (draft)](./specs/tui.md)
- [AI integration notes (legacy)](./docs/agents.md)
- [Roadmap (legacy)](./docs/roadmap.md)

## Migration Notes

This repo has already migrated legacy content into `.ixchel/`:

- ADRs: `.decisions/*.md` → `.ixchel/decisions/dec-*.md`
- Issues: `.tickets/*.md` → `.ixchel/issues/bd-*.md` (hbd-compatible)

For other repos, the minimal manual migration is:

- Move files into the corresponding `.ixchel/<kind>/` folder.
- Ensure each file has an `id: <prefix>-<hex>` frontmatter entry and the
  filename matches the id.
- For hbd issues, normalize `type: <old>` → `type: issue` + `issue_type: <old>`.

## Contributing

See `../CONTRIBUTING.md`.

## License

MIT
