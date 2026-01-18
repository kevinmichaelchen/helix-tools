---
id: dec-6e5c02
type: decision
title: 'ADR-002: Linting and Formatting Toolchain'
status: accepted
date: 2026-01-04
created_at: 2026-01-18T23:33:16Z
updated_at: 2026-01-18T23:33:16Z
created_by: Kevin Chen
tags:
- hbd-ui
- dx
- linting
- tooling
---

> Migrated from `.decisions/002-linting-and-formatting-toolchain.md` into `.ixchel/decisions/`.

# ADR-002: Linting and Formatting Toolchain

**Status:** Accepted\
**Date:** 2026-01-04\
**Deciders:** Kevin Chen\
**Tags:** hbd-ui, dx, linting, tooling

## Context and Problem Statement

hbd-ui is a Svelte 5 + Tauri application that needs a linting and formatting toolchain. The original plan (ticket bd-92268a) specified "Ultracite + Biome," but research revealed significant limitations for Svelte projects.

We need to balance:

1. **Speed** - Fast feedback in editor and CI
2. **Comprehensiveness** - Catch real bugs in Svelte templates
3. **Maintenance** - Minimize tool sprawl

## Decision Drivers

1. **Svelte 5 support** - Must lint `.svelte` files including templates
2. **Performance** - Pre-commit hooks must be fast (<2s)
3. **Rule coverage** - Catch accessibility issues, unused variables, type errors
4. **Editor integration** - VS Code support with inline diagnostics
5. **Future-proofing** - Clear migration path as tools mature

## Research Findings

### Tool Comparison (January 2026)

| Tool       | Speed vs ESLint | Svelte Templates | Rules | Type-Aware       | Stars |
| ---------- | --------------- | ---------------- | ----- | ---------------- | ----- |
| **Oxlint** | 50-100x faster  | `<script>` only  | 645+  | Alpha            | 18K   |
| **Biome**  | 15x faster      | Experimental     | 415   | Custom inference | 23K   |
| **ESLint** | Baseline        | Full via plugin  | 300+  | Full             | 27K   |

### Critical Gap: Svelte Template Linting

**Neither Oxlint nor Biome can lint Svelte-specific syntax:**

| Feature                           | Oxlint | Biome | ESLint + plugin |
| --------------------------------- | ------ | ----- | --------------- |
| `{#if}` / `{#each}` blocks        | No     | No    | **Yes**         |
| `$derived` / `$effect` runes      | No     | No    | **Yes**         |
| Component props validation        | No     | No    | **Yes**         |
| Template accessibility (`a11y-*`) | No     | No    | **Yes**         |
| Unused Svelte imports             | No     | No    | **Yes**         |
| Invalid `bind:` directives        | No     | No    | **Yes**         |

**Only `eslint-plugin-svelte`** (380 stars, actively maintained) provides comprehensive Svelte 5 linting.

### Benchmark Data

| Scenario             | Oxlint     | Biome      | ESLint     |
| -------------------- | ---------- | ---------- | ---------- |
| 1K JS/TS files       | ~20-40ms   | ~60-80ms   | ~1-2s      |
| 10K JS/TS files      | ~200-400ms | ~600-800ms | ~10-20s    |
| Pre-commit (typical) | Instant    | Fast       | Noticeable |

### Ultracite's Role

Ultracite is a **meta-tool** that generates configurations for Biome, ESLint, or Oxlint. It doesn't solve the Svelte linting gap—it just configures whichever underlying tool you choose.

## Considered Options

### Option 1: Biome Only (Original Plan)

```bash
npx ultracite init --linter biome
```

**Pros:**

- Fast (15x ESLint)
- Zero-config via Ultracite
- Single tool for lint + format

**Cons:**

- **No Svelte template linting**
- Misses accessibility issues in templates
- Custom type inference (not TypeScript compiler)

**Verdict:** Rejected - insufficient Svelte coverage

### Option 2: ESLint Only

```bash
npm install -D eslint eslint-plugin-svelte @typescript-eslint/eslint-plugin prettier prettier-plugin-svelte
```

**Pros:**

- Full Svelte 5 support
- Type-aware linting
- Largest ecosystem

**Cons:**

- Slowest option (50-100x slower than Oxlint)
- Requires Prettier for formatting

**Verdict:** Acceptable but suboptimal for DX

### Option 3: Hybrid Oxlint + ESLint (Recommended)

```bash
# Fast linting for JS/TS (90%+ of files)
npm install -D oxlint

# Comprehensive Svelte linting (10% of files)
npm install -D eslint eslint-plugin-svelte

# Formatting
npm install -D prettier prettier-plugin-svelte
```

**Pros:**

- Fast: Oxlint handles 90%+ of files at 50-100x speed
- Comprehensive: ESLint catches Svelte-specific issues
- Clear separation: Each tool does what it's best at

**Cons:**

- Two linters to maintain
- Slightly more complex CI config

**Verdict:** Recommended

### Option 4: Wait for Biome/Oxlint Svelte Support

**Status of Svelte support:**

- Biome: Multiple open issues, no timeline
- Oxlint: Open issues for Vue/Svelte, no timeline

**Verdict:** Not viable for production use today

## Decision

**Adopt Option 3: Hybrid Oxlint + ESLint**

### Rationale

1. **Speed where it matters**: Oxlint lints JS/TS files (typically 80-90% of codebase) at 50-100x ESLint speed
2. **Comprehensive where needed**: ESLint only runs on `.svelte` files, catching template issues
3. **Combined time still fast**: Even with both tools, faster than ESLint-only
4. **Clear migration path**: When Biome/Oxlint adds Svelte support, drop ESLint

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         package.json scripts                     │
│                                                                  │
│  "lint": "oxlint . && eslint --ext .svelte src/"                │
│  "lint:fix": "oxlint --fix . && eslint --fix --ext .svelte src/"│
│  "format": "prettier --write ."                                  │
└─────────────────────────────────────────────────────────────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
        v                      v                      v
┌───────────────┐    ┌─────────────────┐    ┌────────────────┐
│    Oxlint     │    │     ESLint      │    │    Prettier    │
│               │    │                 │    │                │
│ *.ts, *.js    │    │ *.svelte only   │    │ All files      │
│ *.tsx, *.jsx  │    │                 │    │                │
│               │    │ eslint-plugin-  │    │ prettier-      │
│ 645+ rules    │    │ svelte          │    │ plugin-svelte  │
│ Type-aware    │    │                 │    │                │
│ 50-100x fast  │    │ Template lint   │    │ Consistent     │
└───────────────┘    └─────────────────┘    └────────────────┘
```

## Configuration

### oxlint.json

```json
{
  "$schema": "https://raw.githubusercontent.com/oxc-project/oxc/main/npm/oxlint/configuration_schema.json",
  "rules": {
    "no-unused-vars": "error",
    "no-console": "warn",
    "eqeqeq": "error"
  },
  "ignorePatterns": [
    "**/*.svelte",
    "node_modules",
    ".svelte-kit",
    "build"
  ],
  "plugins": {
    "typescript": true,
    "unicorn": true,
    "import": true
  }
}
```

### eslint.config.js

```javascript
import svelte from 'eslint-plugin-svelte';
import svelteParser from 'svelte-eslint-parser';
import tsParser from '@typescript-eslint/parser';

export default [
  // Only lint Svelte files - Oxlint handles the rest
  {
    files: ['**/*.svelte'],
    plugins: {
      svelte,
    },
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: tsParser,
        extraFileExtensions: ['.svelte'],
      },
    },
    rules: {
      // Svelte-specific rules
      ...svelte.configs.recommended.rules,
      'svelte/no-unused-svelte-ignore': 'error',
      'svelte/valid-compile': 'error',
      'svelte/no-at-html-tags': 'warn',
      
      // Accessibility
      'svelte/a11y-click-events-have-key-events': 'error',
      'svelte/a11y-missing-attribute': 'error',
      'svelte/a11y-missing-content': 'error',
      
      // Svelte 5 runes
      'svelte/valid-prop-names-in-kit-pages': 'error',
      'svelte/no-reactive-reassign': 'error',
    },
  },
];
```

### prettier.config.js

```javascript
export default {
  plugins: ['prettier-plugin-svelte'],
  overrides: [
    {
      files: '*.svelte',
      options: {
        parser: 'svelte',
      },
    },
  ],
  singleQuote: true,
  trailingComma: 'es5',
  printWidth: 100,
  tabWidth: 2,
  semi: true,
};
```

### package.json Scripts

```json
{
  "scripts": {
    "lint": "oxlint . && eslint --ext .svelte src/",
    "lint:fix": "oxlint --fix . && eslint --fix --ext .svelte src/",
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "check": "npm run lint && npm run format:check && svelte-check"
  }
}
```

### VS Code Settings (.vscode/settings.json)

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[svelte]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "eslint.validate": ["svelte"],
  "oxc.enable": true,
  "oxc.lint.enable": true
}
```

### Pre-commit Hook (.husky/pre-commit)

```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

# Fast: Oxlint on staged JS/TS files
npx oxlint $(git diff --cached --name-only --diff-filter=ACMR | grep -E '\.(js|ts|jsx|tsx)$' | tr '\n' ' ')

# Comprehensive: ESLint on staged Svelte files  
npx eslint $(git diff --cached --name-only --diff-filter=ACMR | grep -E '\.svelte$' | tr '\n' ' ')

# Format check
npx prettier --check $(git diff --cached --name-only --diff-filter=ACMR | tr '\n' ' ')
```

## Migration Path

### Phase 1: Current (Hybrid)

- Oxlint for JS/TS
- ESLint for Svelte
- Prettier for formatting

### Phase 2: When Biome/Oxlint Supports Svelte

- Monitor [Biome Svelte issues](https://github.com/biomejs/biome/issues?q=svelte)
- Monitor [Oxlint Vue/Svelte issues](https://github.com/oxc-project/oxc/issues/15761)
- When support is stable, evaluate migration

### Phase 3: Single Tool (Future)

- Replace Oxlint + ESLint with single tool
- Replace Prettier with Biome/Oxfmt if Svelte formatting works
- Simplify configuration

## Consequences

### Positive

- **Fast feedback**: Oxlint provides near-instant linting for most files
- **Comprehensive coverage**: ESLint catches Svelte-specific issues
- **Accessibility**: Template a11y rules prevent common issues
- **Type safety**: Both tools support TypeScript
- **Clear separation**: Each tool does what it's best at

### Negative

- **Two linters**: More tools to configure and maintain
- **Potential overlap**: Some rules may fire in both tools (mitigated by ignore patterns)
- **CI complexity**: Two lint steps instead of one

### Risks and Mitigations

| Risk                             | Likelihood | Impact | Mitigation                        |
| -------------------------------- | ---------- | ------ | --------------------------------- |
| Rule conflicts between tools     | Low        | Low    | Oxlint ignores `.svelte` files    |
| eslint-plugin-svelte deprecation | Very Low   | High   | Official Svelte team plugin       |
| Oxlint breaking changes          | Medium     | Low    | Pin versions, test before upgrade |

## Dependencies

```json
{
  "devDependencies": {
    "oxlint": "^0.15.0",
    "eslint": "^9.39.0",
    "eslint-plugin-svelte": "^3.5.0",
    "svelte-eslint-parser": "^0.43.0",
    "@typescript-eslint/parser": "^8.0.0",
    "prettier": "^3.4.0",
    "prettier-plugin-svelte": "^3.4.0"
  }
}
```

## Update to Existing Ticket

**Ticket bd-92268a** should be updated:

**Before:**

> Setup Ultracite + Biome linting

**After:**

> Setup hybrid linting: Oxlint + ESLint + Prettier
>
> - Oxlint for JS/TS files (fast, comprehensive)
> - ESLint with eslint-plugin-svelte for .svelte files (template linting, a11y)
> - Prettier with prettier-plugin-svelte for formatting
>
> See ADR-002 for rationale.

## Related Decisions

- ADR-001: UI Data Rendering Strategy
- Ticket bd-92268a: Setup linting (to be updated)
- Ticket bd-6f8f77: Update CI for UI linting

## References

- [Oxlint Documentation](https://oxc.rs/docs/guide/usage/linter.html)
- [Oxlint Benchmarks](https://oxc.rs/docs/guide/benchmarks)
- [eslint-plugin-svelte](https://github.com/sveltejs/eslint-plugin-svelte)
- [Biome Language Support](https://biomejs.dev/internals/language-support/)
- [Biome Svelte Discussion](https://github.com/biomejs/biome/discussions/136)
