# Welcome to Marrow (Browser Demo)

You're looking at a **mock workspace** running entirely in your browser.
File reads and writes go to an in-memory map, not to disk — refresh the
page and your edits reset. Use this to try the editor, panes, slash menu,
and wiki-link / transclusion features without spinning up Tauri.

## Try these

- Open [[quick-start]] for a markdown syntax tour
- Open [[wiki-graph]] to see backlinks light up
- Look at [[has-unresolved]] — it has a deliberately broken `[[link]]`
- Embed the meeting note below by typing `![[2025-04-28]]` somewhere

## Embedded meeting

![[2025-04-28]]

## What works in browser mode

- Markdown editing (Milkdown WYSIWYG **and** raw CodeMirror via `⌘/`)
- Tabs, splits, drag-and-drop reordering
- Wiki links, transclusion (text content)
- Outline panel (widen this pane past 960px)
- Breadcrumb, edited-time, view-mode toggle
- Backlinks panel (`⌘J` to open the bottom panel)

## What doesn't (yet)

- Full-text search returns empty — Phase 2
- Image paste, PDF preview — Phase 2/3
- Git status, history snapshots — stubbed to safe defaults
- File watcher events — DevPanel will let you trigger them manually later
