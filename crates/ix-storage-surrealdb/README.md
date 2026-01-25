# ix-storage-surrealdb

SurrealDB-backed storage adapter for [Ixchel](https://github.com/kevinmichaelchen/ixchel).

## Features

- Multi-model database with native graph and vector support
- Embedded mode (no external server required)
- Full-text and semantic search capabilities

## Usage

This crate is used internally by `ix-app` when the storage backend is configured as `surrealdb`.

```yaml
# .ixchel/config.yaml
storage:
  backend: surrealdb
```

## License

MIT
