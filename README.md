# Ixchel

![Ixchel](https://live.staticflickr.com/65535/55049015772_351f0dda11_z.jpg)

AI-native developer tools powered by [HelixDB][helixdb]. Git-first, offline-first, agent-friendly.

**[Documentation][docs]** · **[Getting Started][getting-started]** · **[Architecture][architecture]**

## Tools

| Tool                 | Description                        | Status |
| -------------------- | ---------------------------------- | ------ |
| **[ixchel][ixchel]** | Git-first knowledge weaving system | MVP    |

Future tools and experiments are tracked as Ixchel ideas under `.ixchel/ideas/`.

## Quick Start

```bash
# Clone and build
git clone https://github.com/kevinmichaelchen/ixchel.git
cd ixchel
cargo build --release
```

## Try Ixchel

Ixchel is a git-first, Markdown-canonical knowledge weaving system.

```bash
cargo install --path apps/ix-cli
cd your-project

ixchel init
ixchel create decision "Use PostgreSQL for primary storage"
ixchel sync
ixchel search "database performance"
```

## Why Ixchel?

- **Git-first** — Data lives in Markdown files, not a database
- **Offline-first** — Local embeddings, no server required
- **AI-native** — `--json` output, semantic search, agent tracking

## Platform Support

| Platform                      | Status               |
| ----------------------------- | -------------------- |
| macOS (Intel & Apple Silicon) | ✅ Supported         |
| Linux (x86_64 & ARM64)        | ✅ Supported         |
| Windows                       | ❌ Not yet supported |

Windows support is blocked on rewriting the daemon's IPC layer from Unix sockets
to named pipes. Contributions welcome!

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

[docs]: https://kevinmichaelchen.github.io/ixchel
[getting-started]: https://kevinmichaelchen.github.io/ixchel/docs/getting-started
[architecture]: https://kevinmichaelchen.github.io/ixchel/docs/architecture
[config]: https://kevinmichaelchen.github.io/ixchel/docs/configuration

<!-- Tools -->

[ixchel]: ./apps/ix-cli/

<!-- Project -->

[contributing]: ./CONTRIBUTING.md
[license]: ./LICENSE

<!-- External -->

[helixdb]: https://github.com/HelixDB/helix-db
