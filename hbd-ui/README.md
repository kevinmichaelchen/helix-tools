# hbd-ui

A 3D task graph visualizer for [hbd](../hbd), built with Svelte 5, SvelteKit, and [Threlte](https://threlte.xyz/).

## Features

- **3D Graph Visualization**: View your task dependencies as an interactive 3D graph
- **Dual Layout Modes**: Toggle between hierarchical (default) and force-directed layouts
- **Issue Navigation**: Scrollable panel to search and filter issues
- **Click-to-Focus**: Click any node or issue card to focus the camera
- **Status Colors**: Visual indicators for open, in-progress, blocked, and closed issues
- **Demo Mode**: Works without a real hbd repository for testing

## Quick Start

```bash
cd hbd-ui
bun install
bun run dev
```

Open http://localhost:5173 in your browser.

## Usage

### With an hbd Repository

Run from any directory containing a `.ixchel/issues/` folder (or a parent of one):

```bash
cd your-project  # Must have .ixchel/ from `hbd init`
cd path/to/hbd-ui
bun run dev
```

The UI will auto-detect the nearest `.ixchel/issues/` directory and load issues via `hbd list --json`.

### Demo Mode

If no `.ixchel/issues/` directory is found, or to force demo mode:

```
http://localhost:5173?demo=true
```

## Controls

- **Orbit**: Click and drag to rotate the camera
- **Zoom**: Scroll to zoom in/out
- **Pan**: Right-click and drag to pan
- **Select**: Click a node or issue card to select it
- **Toggle Layout**: Use the toolbar to switch between Hierarchical and Force layouts
- **Refresh**: Click the refresh button to reload issues from hbd

## Visual Legend

### Node Colors (Status)

- Gray: Open
- Yellow: In Progress
- Red: Blocked
- Green: Closed

### Node Shapes (Type)

- Box: Epic
- Sphere: All other types

### Node Size (Priority)

- Largest: P0 (Critical)
- Smallest: P4 (Backlog)

### Edge Colors (Dependency Type)

- Red: Blocks
- Orange: Waits For
- Gray: Related

## Development

```bash
bun run dev      # Start dev server
bun run build    # Build for production
bun run preview  # Preview production build
bun run check    # Type check
```

## Architecture

```
src/
├── lib/
│   ├── components/
│   │   ├── three/           # 3D Threlte components
│   │   │   ├── Scene.svelte
│   │   │   ├── TaskGraph.svelte
│   │   │   ├── TaskNode.svelte
│   │   │   └── DependencyEdge.svelte
│   │   ├── ui/              # 2D UI components
│   │   │   ├── IssuePanel.svelte
│   │   │   ├── IssueCard.svelte
│   │   │   ├── DetailPanel.svelte
│   │   │   └── Toolbar.svelte
│   │   └── Canvas.svelte
│   ├── services/
│   │   ├── hbd-client.ts    # Shell integration with hbd CLI
│   │   └── demo-data.ts     # Sample data for demo mode
│   └── types/
│       └── issue.ts         # TypeScript types
└── routes/
    ├── +layout.svelte
    ├── +page.server.ts      # Server-side data loading
    └── +page.svelte         # Main page
```

## Requirements

- [Bun](https://bun.sh/) runtime
- [hbd](../hbd) CLI in PATH (for non-demo mode)
- Modern browser with WebGL support
