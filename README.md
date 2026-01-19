# Helix Tools

AI-native developer tools powered by [HelixDB][helixdb]. Git-first, offline-first, agent-friendly.

**[Documentation][docs]** · **[Getting Started][getting-started]** · **[Architecture][architecture]**

## Tools

| Tool                         | Description                         | Status     |
| ---------------------------- | ----------------------------------- | ---------- |
| **[hbd][hbd]**               | Git-first issue tracker             | Active     |
| **[ixchel][ixchel]**         | Git-first knowledge weaving system  | MVP        |
| **[helix-docs][helix-docs]** | Documentation cache for AI research | Scaffolded |
| **[helix-map][helix-map]**   | Codebase structure indexer          | PoC        |
| **[helix-repo][helix-repo]** | Repository clone manager            | Scaffolded |

## Quick Start

```bash
# Clone and build
git clone https://github.com/kevinmichaelchen/helix-tools.git
cd helix-tools
cargo build --release

# Try hbd (the most complete tool)
cargo install --path hbd
cd your-project
hbd init
hbd create "My first issue" --type task
```

## Try Ixchel

Ixchel is a git-first, Markdown-canonical knowledge weaving system.

```bash
cargo install --path ix-cli
cd your-project

ixchel init
ixchel create decision "Use PostgreSQL for primary storage"
ixchel sync
ixchel search "database performance"
```

## Why Helix?

- **Git-first** — Data lives in Markdown files, not a database
- **Offline-first** — Local embeddings, no server required
- **AI-native** — `--json` output, semantic search, agent tracking

## Learn More

| Resource                     | Description       |
| ---------------------------- | ----------------- |
| [Documentation][docs]        | Full docs site    |
| [Configuration][config]      | How settings work |
| [Architecture][architecture] | System design     |
| [Contributing][contributing] | How to help       |

## License

[MIT][license]

<!-- Docs -->

[docs]: https://kevinmichaelchen.github.io/helix-tools
[getting-started]: https://kevinmichaelchen.github.io/helix-tools/docs/getting-started
[architecture]: https://kevinmichaelchen.github.io/helix-tools/docs/architecture
[config]: https://kevinmichaelchen.github.io/helix-tools/docs/configuration

<!-- Tools -->

[ixchel]: ./ix-cli/
[hbd]: ./hbd/
[helix-docs]: ./helix-docs/
[helix-map]: ./helix-map/
[helix-repo]: ./helix-repo/

<!-- Project -->

[contributing]: ./CONTRIBUTING.md
[license]: ./LICENSE

<!-- External -->

[helixdb]: https://github.com/HelixDB/helix-db
