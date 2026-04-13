# Marrow

A local-first, multi-modal note-taking desktop app. Drop a folder in, browse it, edit markdown in a Notion-style WYSIWYG with `[[wiki-links]]` and `![[transclusions]]`, and preview images, video, audio, PDFs, and code in-app. No database, no cloud — your files stay files.

Built with **Tauri v2 + SvelteKit + Svelte 5 + Milkdown**.

## Philosophy

- **Everything is file.** Drop a folder in, its `.md` files are the source of truth. Anything else can still open them.
- **Local-first.** No sync, no server, no vendor lock-in.
- **Multi-modal.** Markdown, images, video, audio, PDF, and code live side-by-side in the same workspace.

## Status

v1.5 walking-skeleton complete. The app is daily-driver-ready as a personal notes vault.

### Done

- **Workspace**: drag a folder onto the window, lazy-loaded file tree, multi-tab editing with hide-don't-unmount switching, left/right split panes (max 2), tab drag-and-drop between panes.
- **Markdown editor (Milkdown)**: CommonMark + GFM, history, slash command menu (`/`), HTML clipboard paste, list indent (`Tab`/`Shift+Tab`), Prism syntax highlighting in code blocks (light/dark theme aware).
- **Wiki-links**: `[[target]]` with autocomplete, click to navigate (plain / Cmd+click / Cmd+Shift+click variants), unresolved-link styling, basename-based resolution.
- **Transclusion**: `![[target]]` embeds the target file's content inline via markdown-it (depth-limited to 5, live-updates on target change).
- **Backlink panel**: bottom panel (`Cmd+J`) with Backlinks and Unresolved tabs, reverse index built from wiki-link scan, incremental updates on fs-events.
- **File operations**: right-click context menu in the file tree — new file / new folder / delete / folder rename / file rename with wiki-link refactor preview.
- **File rename refactor**: rename a `.md` file → preview shows affected files → confirm → all `[[refs]]` and `![[refs]]` in other files are rewritten in place.
- **Image paste**: paste an image (e.g. screenshot) → written to `attachments/`, standard markdown image inserted.
- **In-app previews**: images, video, audio, PDF (PDF.js, virtualized pages), text/code (CodeMirror 6 read-only with language detection and one-dark theme).
- **Quick-open**: `Cmd+P` fuzzy search across all previewable files in the workspace.
- **Conflict resolution**: external file changes during edit show a 3-way modal (reload / keep mine / see diff via jsdiff).
- **External file watching**: `notify` recursive watcher with own-write filtering — external edits auto-reload non-dirty tabs.
- **Autosave**: 800ms debounce + `Cmd+S`, with mtime conflict guard.
- **Keyboard shortcuts**: `Cmd+S/W/P/J/\\/1/2/Shift+]/Shift+[`.

### Not yet

- Section transclusion (`![[file#header]]`) — full file only
- `[[name|alias]]` syntax
- Tree split (>2 panes), drag-to-create-split
- Backlink graph view
- Full-text search, tag index
- Transactional rename rollback (current is best-effort with error list)

## Getting Started

### Prerequisites
- [Node.js](https://nodejs.org/) + [Yarn](https://yarnpkg.com/)
- [Rust toolchain](https://rustup.rs/) (Edition 2024, Rust 1.85+)

### Install and run
```bash
yarn install
yarn tauri dev
```

Starts Vite on `http://localhost:1620` and launches the Tauri desktop shell.

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
│   ├── +layout.svelte                  # drawer + sidebar + drag-drop + listener init
│   ├── +page.svelte                    # workspace shell or empty state
│   └── Sidebar.svelte                  # workspace header + FileTree host
├── lib/
│   ├── workspace/
│   │   ├── workspace.svelte.ts         # central runes store (panes, tabs, fileIndex, splitRatio)
│   │   ├── types.ts                    # Tab, Pane, FileMeta, FsEventPayload, ReadResult
│   │   ├── fileKind.ts                 # extension → FileKind classifier
│   │   ├── tauri.ts                    # typed invoke wrappers
│   │   ├── fsEvents.ts                 # listen("fs-event") → tree / index / backlinks / transclusion
│   │   ├── shortcuts.svelte.ts         # global keydown + tabSaveRegistry
│   │   └── backlinkIndex.svelte.ts     # reverse [[ref]] index, full + incremental
│   ├── editor/milkdown/
│   │   ├── MilkdownEditor.svelte       # mount-once Milkdown host
│   │   ├── slashCommand.ts             # / slash menu (h1/h2/h3, lists, quote)
│   │   ├── prism.ts                    # Prism code highlighting setup (17 languages)
│   │   ├── imagePaste.ts               # image-clipboard $prose plugin
│   │   ├── wikiLink/                   # [[target]] node + inputRule + suggest + load-pass
│   │   └── transclusion/               # ![[target]] node + NodeView + markdown-it renderer
│   ├── viewers/
│   │   ├── MarkdownTab.svelte          # Milkdown + autosave + dirty + reload + conflict trigger
│   │   ├── ImageTab.svelte / VideoTab / AudioTab / UnsupportedTab
│   │   ├── PdfTab.svelte + PdfViewer/  # bundled pdfjs-dist, virtualized pages
│   │   └── TextTab.svelte + cm/        # CodeMirror 6 read-only with language + theme
│   ├── panes/
│   │   ├── PaneContainer.svelte        # flex split with splitRatio
│   │   ├── PaneResizer.svelte          # drag handle between panes
│   │   ├── Pane.svelte / TabBar / Tab / TabBody
│   ├── tree/
│   │   ├── FileTree.svelte / FileTreeNode.svelte
│   │   ├── treeState.svelte.ts         # lazy expand
│   │   ├── treeOps.ts                  # new file / folder / delete / folder rename
│   │   ├── renameRefactor.ts           # file rename + wiki-link refactor service
│   │   ├── RenameModal.svelte          # 2-stage rename UI
│   │   └── NamePromptModal.svelte      # shared name-input modal
│   ├── panels/
│   │   ├── BottomPanel.svelte          # Cmd+J resizable bottom panel
│   │   ├── BacklinksTab.svelte / UnresolvedTab.svelte
│   │   └── bottomPanelState.svelte.ts
│   ├── quickopen/
│   │   ├── QuickOpen.svelte            # Cmd+P modal
│   │   └── fuzzy.ts                    # subsequence ranker
│   ├── conflict/
│   │   ├── ConflictModal.svelte        # 3-way reload/keep/diff
│   │   └── conflictState.svelte.ts
│   ├── components/ui/
│   │   ├── ContextMenu.svelte          # right-click menu (keyboard navigable)
│   │   ├── SimpleModal.svelte
│   │   └── ...DaisyUI wrappers
│   ├── stores/toastStore.svelte.ts
│   └── utils/
│       ├── tauriUtils.ts               # safeConvertFileSrc
│       └── debounce.ts
├── app.html
├── app.css
└── app.d.ts                            # vite/client reference for ?url imports

src-tauri/
├── src/
│   ├── main.rs                         # binary → marrow_lib::run()
│   ├── lib.rs                          # command registration + WatcherState managed state
│   ├── commands/
│   │   ├── workspace.rs                # 11 file-ops commands
│   │   └── dialog.rs                   # open_directory_dialog
│   └── core/
│       ├── fs_watch.rs                 # notify recursive watcher + recently-written set
│       └── dialog_handler.rs           # rfd folder picker
├── Cargo.toml                          # name = "marrow", lib = "marrow_lib"
└── tauri.conf.json                     # Marrow / com.marrow.app
```

## Architecture Notes

- **State** lives in a single runes store (`workspace.svelte.ts`) holding `info / panes / activePaneId / fileIndex / splitRatio`. ProseMirror / CodeMirror / PDFDocumentProxy instances are **never** put in `$state` — imperative handles live in plain `let` bindings (or `$state.raw()`) to avoid Svelte's proxy deep-observing them.
- **Tab switching** uses hide-don't-unmount (`display: none` on inactive tabs) so undo stacks, cursors, and scroll positions survive. The only place `{#key}` rebuilds an editor is when an external file change reloads a non-dirty markdown tab.
- **Markdown round-trip** uses Milkdown (markdown-first WYSIWYG on ProseMirror + remark) — no HTML/JSON intermediate, so `.md` stays clean. Wiki-links and transclusions are custom ProseMirror nodes with `parseMarkdown.match: () => false` plus a load-time pass that converts text occurrences to nodes after parse.
- **Save↔fs-watch loop** is broken on the Rust side: every mutation command calls `WatcherState::note_own_write` before writing, and the 150ms debouncer filters paths in the recently-written set (500ms TTL, canonicalized).
- **Global keyboard handling** is a single capture-phase listener in `shortcuts.svelte.ts`, registered once in `+layout.svelte`. Per-tab `<svelte:window onkeydown>` is forbidden — `Cmd+S` routes through `tabSaveRegistry`.
- **Wiki-link / transclusion regex coexistence**: wiki-link load-pass and indexer skip matches preceded by `!`. Transclusion load-pass runs **before** wiki-link load-pass on doc mount.
- **Rename refactor safety**: `fs::rename` runs **first** (so the new basename exists on disk), then refs are rewritten best-effort. Partial failure leaves unresolved links rather than dangling refs.

## Tech Stack

| Layer      | Technology       | Version      |
|------------|------------------|--------------|
| Desktop    | Tauri            | v2           |
| Frontend   | SvelteKit        | v2           |
| UI         | Svelte           | v5 (runes)   |
| Editor     | Milkdown         | v7           |
| Code edit  | CodeMirror       | v6           |
| Code highlight | refractor (Prism) | v5     |
| PDF        | pdfjs-dist       | v5           |
| Embed render | markdown-it    | v14          |
| Diff       | jsdiff           | v8           |
| Styling    | Tailwind CSS     | v3           |
| Components | DaisyUI          | v4           |
| Backend    | Rust             | Edition 2024 |

## Adding a Tauri Command

1. Add the function in `src-tauri/src/commands/workspace.rs` (or a new module). Take `state: State<'_, WatcherState>` if it mutates the filesystem and call `state.note_own_write()` for every touched path.
2. Register it in `src-tauri/src/lib.rs` inside `generate_handler!`.
3. Call it from the frontend via a typed wrapper in `src/lib/workspace/tauri.ts`.
