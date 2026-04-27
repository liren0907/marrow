# Demo Workspace

This folder is bundled at build time into the browser mock backend. When
`yarn dev` runs Vite without Tauri, the app boots into this workspace
automatically.

Files here exist to exercise editor features visually:

- `index.md` — landing page with wiki links + transclusion
- `notes/quick-start.md` — markdown syntax cheat sheet
- `notes/wiki-graph.md` — multiple outgoing links + one unresolved
- `meetings/2025-04-28.md` — transclusion target
- `tags-demo.md` — `#tag` indexing
- `has-unresolved.md` — single unresolved link
- `code-samples.md` — syntax highlighting

To add a file: drop a new `.md` in here and reload `yarn dev`. The Vite
glob in `../demoData.ts` picks it up automatically.

Edits made in browser mode are **not** persisted — refreshing the page
resets to whatever's bundled here. (Phase 2 may add a localStorage layer.)
