# DOCS SITE KNOWLEDGE BASE

**Parent:** See root `../AGENTS.md` for project context

## OVERVIEW

Next.js 16 documentation site using Fumadocs. Static export to GitHub Pages.

## STRUCTURE

```
docs/
├── app/
│   ├── (home)/           # Landing page with animated cards
│   ├── docs/             # Documentation layout + dynamic routing
│   ├── api/search/       # Static search endpoint
│   └── og/docs/          # OG image generation
├── content/docs/         # MDX content files
├── lib/                  # Source loader + shared layout config
└── source.config.ts      # Fumadocs MDX configuration
```

## WHERE TO LOOK

| Task | Location |
|------|----------|
| Add docs page | `content/docs/*.mdx` + update `meta.json` |
| Modify navigation | `content/docs/meta.json` (hierarchical) |
| Change branding | `lib/layout.shared.tsx` |
| Custom MDX components | `mdx-components.tsx` |
| Styling/theme | `app/global.css` (Tailwind v4) |
| Build config | `next.config.mjs` + `source.config.ts` |

## CONVENTIONS

- **Package manager**: Bun (not npm/yarn)
- **Styling**: Tailwind CSS v4 with custom PostCSS
- **TypeScript**: Strict mode, ESNext target
- **Routing**: App Router with Fumadocs layouts
- **Content**: MDX with YAML frontmatter via Fumadocs collections

## COMMANDS

```bash
bun install              # Install deps
bun run dev              # Dev server :3000
bun run build            # Static export to out/
bun run types:check      # Type checking
```

## NOTES

- **GitHub Pages deploy**: Conditional `basePath` via `GITHUB_ACTIONS` env
- **Lucide icons**: Auto-converted from icon names in frontmatter
- **OG images**: Generated per-page via `/og/docs/[...slug]/route.tsx`
- **LLM endpoint**: `/llms-full.txt` exports processed markdown for AI consumption
