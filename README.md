# md-editor

A local-first, multi-modal note-taking desktop app. Drop a folder in, browse it, edit markdown in a Notion-like WYSIWYG editor, and preview images, video, audio, and (soon) PDFs in-app. No database, no cloud — your files stay files.

Built with **Tauri v2 + SvelteKit + Svelte 5 + Milkdown**.

> **Status:** MVP walking skeleton. See the roadmap below for what's working today and what's coming.

## Philosophy

- **Everything is file.** Drop a folder in, its `.md` files are the source of truth. Anything else can still open it.
- **Local-first.** No sync, no server, no vendor lock-in.
- **Multi-modal.** Markdown, images, video, audio, and PDF live side-by-side in the same workspace.

## Roadmap

### MVP (done)
- Drag a folder onto the window → workspace root
- Lazy-expanding file tree in the sidebar
- Multi-tab editing (single pane), hide-don't-unmount switching
- Markdown via **Milkdown** (CommonMark + GFM + history + listener)
- Autosave (800ms debounce) + `Cmd+S`, with mtime conflict guard
- In-app previews: images, video, audio
- "Open with system" fallback for unsupported types

### v1 (next)
- Slash command menu in the editor
- **PDF.js** viewer (bundled, virtualized pages, zoom)
- CodeMirror 6 read-only preview for text/code
- Left/right split panes (`Cmd+\`, `Cmd+1/2`)
- `[[wiki link]]` — parse, render, click-to-navigate, `[[` autocomplete
- `notify`-based file watcher (external edits auto-reload)
- Quick-open (`Cmd+P`) over `.md` paths
- Keyboard shortcuts: `Cmd+S / W / P / \ / 1 / 2 / Shift+] [`

### v1.5
- Backlink panel, unresolved-link styling
- Context menu: rename / delete / new file / new folder (with wiki-link refactor)
- Tab drag between panes
- Image paste → `attachments/` + `![[...]]`
- Wiki-link transclusion `![[foo]]`

## Getting Started

### Prerequisites
- [Node.js](https://nodejs.org/) + [Yarn](https://yarnpkg.com/)
- [Rust toolchain](https://rustup.rs/) (Edition 2024, Rust 1.85+)

### Install and run
```bash
yarn install
yarn tauri dev
```

Starts Vite on `http://localhost:1420` and launches the Tauri desktop shell.

### Build for production
```bash
yarn tauri build
```

### Type check
```bash
yarn run check         # one-shot
yarn run check:watch   # watch mode
```

## Project Structure

```
src/
├── routes/
│   ├── +layout.svelte              # drawer + drag-drop listener
│   ├── +page.svelte                # workspace or empty-state
│   └── Sidebar.svelte               # workspace header + FileTree host
├── lib/
│   ├── workspace/
│   │   ├── workspace.svelte.ts      # central runes store (panes / tabs)
│   │   ├── types.ts                 # Tab, Pane, DirEntry, FileKind
│   │   ├── fileKind.ts              # extension → FileKind
│   │   └── tauri.ts                 # typed invoke wrappers
│   ├── editor/milkdown/
│   │   └── MilkdownEditor.svelte    # mount-once Milkdown host
│   ├── viewers/
│   │   ├── MarkdownTab.svelte       # Milkdown + autosave + dirty
│   │   ├── ImageTab.svelte
│   │   ├── VideoTab.svelte
│   │   ├── AudioTab.svelte
│   │   └── UnsupportedTab.svelte
│   ├── panes/
│   │   ├── PaneContainer.svelte
│   │   ├── Pane.svelte
│   │   ├── TabBar.svelte / Tab.svelte / TabBody.svelte
│   ├── tree/
│   │   ├── FileTree.svelte
│   │   ├── FileTreeNode.svelte
│   │   └── treeState.svelte.ts      # lazy expand + SvelteMap/Set
│   ├── components/ui/               # DaisyUI wrappers
│   ├── stores/toastStore.svelte.ts
│   └── utils/
│       ├── tauriUtils.ts            # safeConvertFileSrc
│       └── debounce.ts
├── app.html
└── app.css

src-tauri/
├── src/
│   ├── main.rs
│   ├── lib.rs                       # command registration
│   ├── commands/
│   │   ├── workspace.rs             # open/list/read/write file ops
│   │   └── dialog.rs                # open_directory_dialog
│   └── core/dialog_handler.rs       # rfd folder picker
├── Cargo.toml
└── tauri.conf.json
```

## Architecture Notes

- **State** lives in a single runes store (`workspace.svelte.ts`) holding `info / panes / activePaneId`. ProseMirror / editor instances are **never** put in `$state` — imperative handles live inside component-local `let` bindings to avoid Svelte proxy deep-observing them.
- **Tab switching** uses hide-don't-unmount (`display: none` on inactive tabs) so undo stacks, cursors, and scroll positions survive.
- **Markdown round-trip** uses Milkdown (markdown-first WYSIWYG on ProseMirror + remark) — no HTML/JSON intermediate, so `.md` stays clean.
- **Autosave** is debounced (800ms) and a `Cmd+S` press flushes the queue. Writes pass an `expected_mtime` so external edits won't get clobbered silently.

## Tech Stack

| Layer      | Technology       | Version      |
|------------|------------------|--------------|
| Desktop    | Tauri            | v2           |
| Frontend   | SvelteKit        | v2           |
| UI         | Svelte           | v5 (runes)   |
| Editor     | Milkdown         | v7           |
| Styling    | Tailwind CSS     | v3           |
| Components | DaisyUI          | v4           |
| Backend    | Rust             | Edition 2024 |

## Adding a Tauri Command

1. Add the function in `src-tauri/src/commands/workspace.rs` (or a new module).
2. Register it in `src-tauri/src/lib.rs` inside `invoke_handler![]`.
3. Call it from the frontend via a typed wrapper in `src/lib/workspace/tauri.ts`.
