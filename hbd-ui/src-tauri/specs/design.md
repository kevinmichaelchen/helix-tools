# Design

**Crate:** `hbd-ui/src-tauri`\
**Purpose:** Desktop packaging for the `hbd-ui` frontend

## Overview

This crate is a thin Tauri wrapper:

- `src/main.rs` is the entrypoint
- `src/lib.rs` configures the Tauri builder
- `tauri.conf.json` controls bundling and runtime settings

Domain behavior (issue querying, filtering, UI interactions) lives in the
frontend.
