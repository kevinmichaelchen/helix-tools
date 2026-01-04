---
id: bd-39ddc9
title: Initialize Tauri v2 for macOS desktop app
status: open
priority: 1
type: task
created_at: 2026-01-04T05:14:32.933873+00:00
updated_at: 2026-01-04T05:14:32.933873+00:00
created_by: kevinchen
created_by_type: human
labels:
- sunday-jan-5th
- tauri
- desktop
---

Run tauri init in hbd-ui. Configure for SvelteKit SSG (adapter-static, ssr=false, prerender=true). Set frontendDist to build/. Update workspace Cargo.toml to include src-tauri. Configure tauri.conf.json for macOS with app name hbd-ui and window title.
