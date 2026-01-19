# Model Management Guide

**Audience:** Developers using ix-embeddings on Apple Silicon Macs
**Constraint:** Limited disk space (typically 80GB free)

## TL;DR

```bash
# Install the official CLI
pip install -U huggingface_hub

# See what's using disk space
huggingface-cli scan-cache

# Download a model before going offline
huggingface-cli download BAAI/bge-large-en-v1.5

# Free up space
huggingface-cli delete-cache
```

---

## How Models Are Downloaded

When you use ix-embeddings with Candle, models are automatically downloaded
from [Hugging Face Hub][hf-hub] on first use:

```rust
let config = EmbeddingConfig {
    provider: "candle".to_string(),
    model: "BAAI/bge-large-en-v1.5".to_string(),  // Downloaded automatically
    ..Default::default()
};
let embedder = Embedder::with_config(&config)?;
```

Models are cached locally so subsequent runs don't re-download.

---

## Cache Location

All Hugging Face models are stored in:

```
~/.cache/huggingface/hub/
├── models--BAAI--bge-large-en-v1.5/
│   ├── blobs/          # Actual weight files
│   ├── refs/           # Branch/tag references
│   └── snapshots/      # Version snapshots
├── models--sentence-transformers--all-MiniLM-L6-v2/
└── ...
```

**Shared cache:** The `hf-hub` Rust crate and `huggingface-cli` share the same
cache. Models downloaded by either are available to both.

---

## Model Sizes

| Model                                    | Params | Dims | Disk Size |
| ---------------------------------------- | ------ | ---- | --------- |
| `sentence-transformers/all-MiniLM-L6-v2` | 22M    | 384  | ~90 MB    |
| `BAAI/bge-small-en-v1.5`                 | 33M    | 384  | ~130 MB   |
| `BAAI/bge-base-en-v1.5`                  | 109M   | 768  | ~440 MB   |
| `BAAI/bge-large-en-v1.5`                 | 335M   | 1024 | ~1.3 GB   |

**With 80GB free:** You can store ~60 large models. Realistically, you'll use 2-3.

---

## Managing Disk Space

### Check Current Usage

```bash
huggingface-cli scan-cache
```

Output:

```
REPO ID                                   REPO TYPE  SIZE ON DISK  NB FILES  LAST_ACCESSED
----------------------------------------  ---------  ------------  --------  -------------------
BAAI/bge-large-en-v1.5                    model      1.3 GB        5         2 days ago
sentence-transformers/all-MiniLM-L6-v2    model      90.4 MB       7         5 minutes ago

Done in 0.0s. Scanned 2 repo(s) for a total of 1.4 GB.
```

### Delete Specific Models

```bash
# Interactive deletion
huggingface-cli delete-cache

# Or delete a specific model directory
rm -rf ~/.cache/huggingface/hub/models--BAAI--bge-large-en-v1.5
```

### Set Custom Cache Location

If you have an external drive with more space:

```bash
export HF_HUB_CACHE=/Volumes/ExternalDrive/huggingface
```

Add to `~/.zshrc` to persist.

---

## Offline Usage

### Pre-download Models

Before going offline (flight, coffee shop with bad WiFi):

```bash
# Download the model you'll need
huggingface-cli download BAAI/bge-large-en-v1.5

# Verify it's cached
huggingface-cli scan-cache | grep bge-large
```

### Offline Mode

Set this environment variable to prevent network requests:

```bash
export HF_HUB_OFFLINE=1
```

If a model isn't cached, you'll get a clear error instead of a hang.

---

## Troubleshooting

### "Model not found" but it exists on Hugging Face

Check your internet connection, or the model may require authentication:

```bash
# Login to Hugging Face (for gated models)
huggingface-cli login
```

### Slow downloads

Hugging Face Hub can be slow. For large models, consider:

```bash
# Use aria2 for faster parallel downloads
pip install hf-transfer
export HF_HUB_ENABLE_HF_TRANSFER=1
huggingface-cli download BAAI/bge-large-en-v1.5
```

### Cache corruption

If a model fails to load after download:

```bash
# Remove and re-download
rm -rf ~/.cache/huggingface/hub/models--BAAI--bge-large-en-v1.5
huggingface-cli download BAAI/bge-large-en-v1.5
```

---

## Why Not Build Our Own CLI?

We considered building a Rust CLI for model management, but:

1. **`huggingface-cli` is well-maintained** - Official tool, regular updates
2. **Shared cache** - Works seamlessly with our `hf-hub` Rust dependency
3. **Feature-rich** - Auth, transfers, revisions, deduplication
4. **Not our core mission** - ixchel is about developer productivity, not
   model distribution

The Python dependency is minimal (just the CLI, not runtime).

---

## References

- [Hugging Face Hub Documentation][hf-docs]
- [Cache Management Guide][hf-cache]
- [hf-hub Rust Crate][hf-hub-rs]

[hf-hub]: https://huggingface.co/
[hf-docs]: https://huggingface.co/docs/hub/
[hf-cache]: https://huggingface.co/docs/huggingface_hub/guides/manage-cache
[hf-hub-rs]: https://crates.io/crates/hf-hub
