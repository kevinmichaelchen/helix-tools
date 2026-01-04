---
id: bd-39ddc9
title: Initialize Tauri v2 for macOS desktop app
status: closed
priority: 1
type: task
created_at: 2026-01-04T05:14:32.933873+00:00
updated_at: 2026-01-04T19:51:00.000000+00:00
created_by: kevinchen
created_by_type: human
labels:
- sunday-jan-5th
- tauri
- desktop
---

Completed Tauri v2 initialization for hbd-ui desktop app.

## Changes Made

- Installed `@sveltejs/adapter-static` and removed `adapter-node`
- Updated `svelte.config.js` to use adapter-static with SPA fallback
- Created `src/routes/+layout.ts` with `ssr=false` and `prerender=true`
- Ran `cargo tauri init` to generate src-tauri scaffolding
- Updated `hbd-ui/src-tauri/Cargo.toml`:
  - Package name: `hbd-ui`
  - Library name: `hbd_ui_lib`
  - Added `hbd` crate dependency for Tauri-native backend integration
  - Connected to workspace for shared deps and lints
- Updated `hbd-ui/src-tauri/tauri.conf.json`:
  - App identifier: `dev.kevinchen.hbd-ui`
  - Window: 1200x800 default, 800x600 min
  - Build commands configured for bun
- Updated root `Cargo.toml` to include `hbd-ui/src-tauri` in workspace
- Added `tauri`, `tauri:dev`, `tauri:build` scripts to package.json

## Verification

- `cargo check -p hbd-ui` passes
- `bun run check` passes (svelte-check)
