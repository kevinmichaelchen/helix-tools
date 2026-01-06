---
id: 1
title: Test Architecture Decision
status: accepted
date: 2026-01-05
deciders:
  - Alice
  - Bob
tags:
  - testing
  - architecture
---

# Context and Problem Statement

This is a test ADR used for integration testing of adr-search.

## Decision Drivers

- Need to test ADR parsing
- Need to test semantic search
- Need to test delta indexing

## Considered Options

1. Use real ADRs
2. Create fixture ADRs

## Decision

Use fixture ADRs in tests/fixtures/.decisions/

## Consequences

- Easy to test edge cases
- Consistent test results
- No external dependencies
