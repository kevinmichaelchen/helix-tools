# ADR-003: Binary Installation Strategy

**Status:** Proposed\
**Date:** 2026-01-05\
**Deciders:** Kevin Chen\
**Tags:** distribution, dx, installation, ci

## Context and Problem Statement

helix-tools produces CLI binaries (primarily `hbd`) that users need to install. Currently, installation requires cloning the repo and running `cargo build`. We need a frictionless installation experience that:

1. Works for Rust developers (primary audience)
2. Optionally supports non-Rust users
3. Minimizes maintenance burden
4. Provides automatic updates or clear upgrade path

## Decision Drivers

1. **Target audience** - Primarily developers who likely have Rust installed
2. **Maintenance cost** - Scripts and packages require ongoing maintenance
3. **Security** - Avoid controversial patterns like `curl | bash` if possible
4. **Cross-platform** - Linux, macOS (Intel + Apple Silicon), Windows
5. **Discoverability** - Users should find installation instructions easily

## Research Findings

### Installation Methods in the Rust Ecosystem (January 2026)

| Method           | Examples         | Rust Required     | Maintenance | UX     |
| ---------------- | ---------------- | ----------------- | ----------- | ------ |
| `cargo install`  | Most crates      | Yes               | None        | Good   |
| `cargo binstall` | ripgrep, bat, fd | Yes (binstall)    | Low         | Great  |
| Homebrew         | gh, jq           | No                | Medium      | Great  |
| `curl \| bash`   | rustup, beads    | No                | High        | Quick  |
| GitHub Releases  | All of the above | No                | Low         | Manual |
| Nix/nixpkgs      | Many tools       | No (Nix required) | Medium      | Good   |

### cargo-binstall Ecosystem

[cargo-binstall](https://github.com/cargo-bins/cargo-binstall) (2.4K stars) provides:

- Automatic binary detection from GitHub Releases
- Fallback to `cargo install` if no binary available
- Platform detection (OS, architecture, libc)
- Checksum verification
- No changes needed to Cargo.toml (convention-based)

**Popular tools using binstall:** ripgrep, bat, fd, tokei, hyperfine, starship

### Homebrew Considerations

Pros:

- Most familiar to macOS users
- Auto-updates via `brew upgrade`
- Can install without Rust

Cons:

- Requires formula submission to homebrew-core (slow approval) or maintaining a tap
- Formula maintenance on each release
- Linux support via Linuxbrew is less common

### curl | bash Pattern (Beads example)

```bash
curl -fsSL https://raw.githubusercontent.com/steveyegge/beads/main/scripts/install.sh | bash
```

Pros:

- One-liner, familiar to DevOps users
- No Rust required
- Works on any Unix-like system

Cons:

- Security concerns (executing untrusted code)
- Script maintenance: platform detection, error handling, checksums
- No built-in update mechanism
- Windows requires WSL or separate PowerShell script

## Considered Options

### Option 1: cargo-binstall Only

**Installation:**

```bash
# If cargo-binstall is installed
cargo binstall hbd

# Fallback (compiles from source)
cargo install hbd
```

**Requirements:**

- GitHub Actions workflow to build and publish release binaries
- Naming convention: `hbd-{version}-{target}.tar.gz`

**Pros:**

- Zero script maintenance
- Automatic platform detection
- Fallback to source compilation
- Standard Rust ecosystem approach
- Checksum verification built-in

**Cons:**

- Requires Rust toolchain installed
- Extra step to install cargo-binstall itself
- Not suitable for non-developers

### Option 2: cargo-binstall + Homebrew Tap

**Installation:**

```bash
# macOS/Linux (Homebrew)
brew install kevinmichaelchen/tap/hbd

# Rust developers
cargo binstall hbd
```

**Requirements:**

- Same as Option 1, plus:
- Create `homebrew-tap` repository
- Maintain formula (can be automated with `goreleaser` or similar)

**Pros:**

- Best UX for macOS users
- No Rust required for Homebrew path
- Familiar to broad audience

**Cons:**

- Tap maintenance overhead
- Formula updates on each release
- Another repo to maintain

### Option 3: cargo-binstall + Install Script

**Installation:**

```bash
# Quick install (downloads pre-built binary)
curl -fsSL https://helix-tools.dev/install.sh | bash

# Rust developers
cargo binstall hbd
```

**Requirements:**

- Same as Option 1, plus:
- Write and maintain `install.sh` (and `install.ps1` for Windows)
- Handle: platform detection, architecture, checksums, PATH setup, error handling

**Pros:**

- One-liner for non-Rust users
- Full control over installation experience
- Works without any prerequisites

**Cons:**

- Script maintenance burden
- Security optics (curl | bash)
- Must handle edge cases (ARM64, musl, etc.)
- Separate Windows script needed

### Option 4: GitHub Releases Only (Minimal)

**Installation:**

```bash
# Download from releases page
# Manually extract and add to PATH
```

**Pros:**

- Zero maintenance beyond CI builds
- Maximum transparency (users see exactly what they download)

**Cons:**

- Poor UX: manual download, extraction, PATH setup
- No update mechanism
- Not suitable for broad adoption

### Option 5: cargo-binstall + Nix Flake

**Installation:**

```bash
# Nix users
nix run github:kevinmichaelchen/helix-tools#hbd

# Rust developers
cargo binstall hbd
```

**Requirements:**

- Add `flake.nix` to repository
- Define package outputs

**Pros:**

- Reproducible builds
- Popular in certain developer communities
- Can include development shells

**Cons:**

- Nix has steep learning curve
- Smaller audience than Homebrew
- Flake maintenance

## Comparison Matrix

| Criteria           | Option 1 | Option 2     | Option 3   | Option 4 | Option 5  |
| ------------------ | -------- | ------------ | ---------- | -------- | --------- |
| **Maintenance**    | Low      | Medium       | High       | Minimal  | Medium    |
| **Rust required**  | Yes      | No (brew)    | No         | No       | No (nix)  |
| **UX (Rust devs)** | Great    | Great        | Great      | Poor     | Great     |
| **UX (non-Rust)**  | N/A      | Great        | Good       | Poor     | Good      |
| **Security**       | Good     | Good         | Moderate   | Good     | Good      |
| **Cross-platform** | Yes      | macOS focus  | Unix focus | Yes      | Yes       |
| **Auto-updates**   | Manual   | brew upgrade | Manual     | Manual   | nix flake |

## Recommendation

**Start with Option 1 (cargo-binstall only)**, then expand based on demand:

### Phase 1: cargo-binstall (Now)

- Set up GitHub Actions to build release binaries for:
  - `x86_64-unknown-linux-gnu`
  - `x86_64-unknown-linux-musl`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`
- Document: `cargo binstall hbd` and `cargo install hbd`
- This covers 90%+ of our target audience (Rust developers)

### Phase 2: Homebrew (If Requested)

- Create `kevinmichaelchen/homebrew-tap` repository
- Add formula for `hbd`
- Automate formula updates via GitHub Actions

### Phase 3: Install Script (If Demanded by Non-Rust Users)

- Only if significant demand from non-developers
- Consider using existing solutions like `cargo-dist` which generates install scripts

## Implementation Notes

### cargo-binstall Auto-Detection

cargo-binstall looks for release assets matching these patterns:

```
{name}-{version}-{target}.tar.gz
{name}-{version}-{target}.zip
{name}-v{version}-{target}.tar.gz
```

Example for `hbd v0.1.0` on Apple Silicon:

```
hbd-0.1.0-aarch64-apple-darwin.tar.gz
```

### Optional: Explicit Binstall Metadata

Add to `hbd/Cargo.toml` for custom URLs or formats:

```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ version }-{ target }{ archive-suffix }"
pkg-fmt = "tgz"
```

### CI Workflow Skeleton

```yaml
name: Release

on:
  push:
    tags: ["v*"]

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          # ... more targets
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }} -p hbd
      - name: Archive
        run: tar -czvf hbd-${{ github.ref_name }}-${{ matrix.target }}.tar.gz -C target/${{ matrix.target }}/release hbd
      - uses: softprops/action-gh-release@v1
        with:
          files: hbd-*.tar.gz
```

## Decision

**Pending** - Awaiting final decision on phased approach.

## References

- [cargo-binstall documentation](https://github.com/cargo-bins/cargo-binstall)
- [cargo-dist](https://github.com/axodotdev/cargo-dist) - Alternative that generates installers
- [Beads install.sh](https://github.com/steveyegge/beads/blob/main/scripts/install.sh)
- [ripgrep releases](https://github.com/BurntSushi/ripgrep/releases) - Example of well-structured releases
