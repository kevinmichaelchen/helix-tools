# Requirements

This document defines requirements for the `hbd-ui` Tauri shell crate.

## 1. Desktop Shell

### US-001: Run the frontend in a desktop window

| ID       | Acceptance Criterion                                                          |
| -------- | ----------------------------------------------------------------------------- |
| AC-001.1 | THE SYSTEM SHALL package the `hbd-ui` frontend as a Tauri desktop application |
| AC-001.2 | THE SYSTEM SHALL launch the UI in a native window using Tauri defaults        |

## 2. Compatibility

### US-002: Minimal coupling

| ID       | Acceptance Criterion                                                            |
| -------- | ------------------------------------------------------------------------------- |
| AC-002.1 | The Tauri shell SHALL keep domain logic in the frontend or `hbd`/Ixchel crates  |
| AC-002.2 | The Tauri shell SHALL avoid embedding project-specific assumptions in Rust code |
