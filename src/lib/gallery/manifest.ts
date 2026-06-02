import type { IconName } from "$lib/components/ui/Icon.svelte";
import { workspace } from "$lib/workspace/workspace.svelte";
import { pathForImport } from "./discovery";

// Curated catalog for the dev-only /gallery, organised the way the user's
// reference project does it: ROUTE (a logical view/page of the app) → semantic
// GROUP (with a coloured rule) → component. Each component carries a blurb and
// a preview spec. This is hand-authored on purpose — it reads far better than
// an auto-glob dump, and it lets us seed store-driven components with realistic
// data instead of showing "listing only".

export type Accent = "primary" | "accent" | "secondary" | "info";
export type PreviewKind =
  | "props" // pure prop-driven — render immediately from the given props
  | "seed" // store-driven — render after the dev-only workspace seed
  | "note"; // heavy/native (Milkdown/PDF/Cytoscape/Excalidraw) — show a note

export interface PreviewExample {
  label?: string;
  // Static props, or a thunk evaluated at render time (after seed) so it can
  // read live seeded workspace data (e.g. the active pane / tab).
  props?: Record<string, unknown> | (() => Record<string, unknown>);
  text?: string; // plain text passed as default children, when the component takes any
}

export interface ManifestItem {
  name: string;
  importPath: string; // "$lib/..."
  kind: PreviewKind;
  blurb: string;
  note?: string; // for kind "note"
  examples?: PreviewExample[]; // for kind "props" | "seed"
}

export interface ManifestGroup {
  label: string;
  accent: Accent;
  items: ManifestItem[];
}

export interface ManifestRoute {
  id: string;
  title: string;
  chip?: string;
  blurb: string;
  groups: ManifestGroup[];
}

export function globPath(item: ManifestItem): string {
  return pathForImport(item.importPath);
}

const noop = (): void => {};

const ICON_NAMES: IconName[] = [
  "plus", "arrow-left", "chevron-right", "chevron-down", "chevron-up", "x",
  "folder-plus", "trash-2", "file-text", "file", "file-code", "pencil", "folder",
  "folder-open", "history", "network", "image", "film", "music", "file-plus",
  "maximize-2", "external-link", "search", "tag", "terminal", "columns-2", "info",
  "circle-check", "triangle-alert", "circle-alert", "rotate-ccw", "move-right",
  "sliders-horizontal", "flask-conical", "settings", "palette", "target", "code",
  "eye", "panel-right-close", "panel-right-open", "panel-left-close",
  "panel-left-open", "keyboard", "brush",
];

export const MANIFEST: ManifestRoute[] = [
  // ── UI primitives (shared, pure prop-driven) ──────────────────────────────
  {
    id: "ui",
    title: "UI primitives",
    chip: "$lib/components/ui",
    blurb: "跨視圖共用的純 prop-driven 元件。不碰 store、不發 Tauri，直接用範例 props 渲染。",
    groups: [
      {
        label: "Primitive",
        accent: "secondary",
        items: [
          {
            name: "Button",
            importPath: "$lib/components/ui/Button.svelte",
            kind: "props",
            blurb: "Flat-gray 按鈕。variant default/ghost、size xs/sm/md。",
            examples: [
              { label: "default", props: { variant: "default" }, text: "Button" },
              { label: "ghost", props: { variant: "ghost" }, text: "Ghost" },
              { label: "xs", props: { size: "xs" }, text: "Extra small" },
              { label: "disabled", props: { disabled: true }, text: "Disabled" },
            ],
          },
          {
            name: "Badge",
            importPath: "$lib/components/ui/Badge.svelte",
            kind: "props",
            blurb: "DaisyUI badge 包裝，含 variant / size。",
            examples: [
              { label: "default", text: "Badge" },
              { label: "primary", props: { variant: "primary" }, text: "Primary" },
              { label: "success", props: { variant: "success" }, text: "Success" },
              { label: "outline", props: { variant: "outline" }, text: "Outline" },
            ],
          },
          {
            name: "Alert",
            importPath: "$lib/components/ui/Alert.svelte",
            kind: "props",
            blurb: "info/success/warning/error 四色提示，可選標題與可關閉。",
            examples: [
              { label: "info", props: { variant: "info", title: "Heads up" }, text: "An informational alert." },
              { label: "success", props: { variant: "success" }, text: "Saved successfully." },
              { label: "warning", props: { variant: "warning" }, text: "Careful now." },
              { label: "error", props: { variant: "error", dismissible: true }, text: "Something broke." },
            ],
          },
          {
            name: "Card",
            importPath: "$lib/components/ui/Card.svelte",
            kind: "props",
            blurb: "含可選標題/圖示的卡片容器，base-100 / base-200 兩色。",
            examples: [
              { label: "titled", props: { title: "Card title", icon: "info" }, text: "Card body content." },
              { label: "base-200", props: { title: "Shaded", variant: "base-200" }, text: "On base-200." },
            ],
          },
          {
            name: "SectionLabel",
            importPath: "$lib/components/ui/SectionLabel.svelte",
            kind: "props",
            blurb: "區段小標(uppercase、字距加寬)。",
            examples: [{ text: "Section label" }],
          },
          {
            name: "TextInput",
            importPath: "$lib/components/ui/TextInput.svelte",
            kind: "props",
            blurb: "文字/數字輸入，bordered / ghost 兩款。",
            examples: [
              { label: "bordered", props: { placeholder: "Type here…" } },
              { label: "ghost", props: { variant: "ghost", placeholder: "Ghost input" } },
            ],
          },
        ],
      },
      {
        label: "Icons & button groups",
        accent: "accent",
        items: [
          {
            name: "IconButton",
            importPath: "$lib/components/ui/IconButton.svelte",
            kind: "props",
            blurb: "圖示按鈕，ghost/soft、可帶 label、active 態。",
            examples: [
              { label: "ghost", props: { icon: "settings", tooltip: "Settings" } },
              { label: "soft", props: { icon: "search", variant: "soft" } },
              { label: "active", props: { icon: "eye", active: true } },
              { label: "with label", props: { icon: "plus", label: "New" } },
            ],
          },
          {
            name: "ToggleButtonGroup",
            importPath: "$lib/components/ui/ToggleButtonGroup.svelte",
            kind: "props",
            blurb: "互斥切換鈕群組(圖示或文字)。",
            examples: [
              {
                label: "icons",
                props: {
                  value: "list",
                  options: [
                    { value: "list", icon: "columns-2", tooltip: "List" },
                    { value: "grid", icon: "network", tooltip: "Grid" },
                  ],
                },
              },
              {
                label: "labels",
                props: {
                  value: "a",
                  options: [
                    { value: "a", label: "One" },
                    { value: "b", label: "Two" },
                  ],
                },
              },
            ],
          },
          {
            name: "Icon",
            importPath: "$lib/components/ui/Icon.svelte",
            kind: "props",
            blurb: `自繪 lucide SVG(不依賴 .svelte，避開 Svelte 5 TS 解析問題)。全部 ${ICON_NAMES.length} 個 IconName：`,
            examples: ICON_NAMES.map((n) => ({ label: n, props: { name: n, size: 22 } })),
          },
        ],
      },
    ],
  },

  // ── workspace (the main / view) ───────────────────────────────────────────
  {
    id: "workspace",
    title: "workspace",
    chip: "/ — 主視圖",
    blurb:
      "Marrow 的主編輯視圖。store-driven 元件由 dev-only seed 餵入(開啟 devmock 假 workspace),不發任何 Tauri 請求。重量級編輯器/viewer 需真實檔案,於此標註而不掛載。",
    groups: [
      {
        label: "Chrome",
        accent: "primary",
        items: [
          { name: "TitleBar", importPath: "$lib/chrome/TitleBar.svelte", kind: "seed", blurb: "頂部標題列(workspace 名 + 路徑)。", examples: [{}] },
          { name: "ActivityBar", importPath: "$lib/chrome/ActivityBar.svelte", kind: "seed", blurb: "最左窄圖示列(Files/Search/Tags/Graph… 切換)。", examples: [{}] },
          { name: "StatusBar", importPath: "$lib/chrome/StatusBar.svelte", kind: "seed", blurb: "底部狀態列(watcher / git / 筆記數 / pane·tab / 格式)。", examples: [{}] },
          {
            name: "Breadcrumb",
            importPath: "$lib/panes/Breadcrumb.svelte",
            kind: "seed",
            blurb: "active tab 的路徑麵包屑。需 pane prop。",
            examples: [{ props: () => ({ pane: workspace.activePane }) }],
          },
        ],
      },
      {
        label: "File tree",
        accent: "accent",
        items: [
          { name: "FileTree", importPath: "$lib/tree/FileTree.svelte", kind: "seed", blurb: "workspace 檔案總管根容器(讀 fileIndex + 懶展開)。", examples: [{}] },
        ],
      },
      {
        label: "Panes & tabs",
        accent: "secondary",
        items: [
          {
            name: "TabBar",
            importPath: "$lib/panes/TabBar.svelte",
            kind: "seed",
            blurb: "分頁列 + 拖放落點。需 pane prop。",
            examples: [{ props: () => ({ pane: workspace.activePane }) }],
          },
          {
            name: "Tab",
            importPath: "$lib/panes/Tab.svelte",
            kind: "seed",
            blurb: "單一可拖曳分頁。需 tab / paneId / active。",
            examples: [
              {
                props: () => {
                  const p = workspace.activePane;
                  return { tab: p.tabs[0], paneId: p.id, active: true };
                },
              },
            ],
          },
          { name: "PaneResizer", importPath: "$lib/panes/PaneResizer.svelte", kind: "seed", blurb: "兩個 pane 之間的拖曳分隔把手。", examples: [{}] },
        ],
      },
      {
        label: "Panels (Cmd+J)",
        accent: "info",
        items: [
          { name: "BacklinksTab", importPath: "$lib/panels/BacklinksTab.svelte", kind: "seed", blurb: "含 [[currentFile]] 的反向連結清單。", examples: [{}] },
          { name: "UnresolvedTab", importPath: "$lib/panels/UnresolvedTab.svelte", kind: "seed", blurb: "全 workspace 未解析 [[refs]],依來源分組。", examples: [{}] },
          { name: "TagsTab", importPath: "$lib/panels/TagsTab.svelte", kind: "seed", blurb: "標籤面板。", examples: [{}] },
          { name: "OutlineTab", importPath: "$lib/panels/OutlineTab.svelte", kind: "seed", blurb: "目前 markdown 的標題大綱。", examples: [{}] },
        ],
      },
      {
        label: "Editor & viewers",
        accent: "primary",
        items: [
          { name: "MarkdownTab", importPath: "$lib/viewers/MarkdownTab.svelte", kind: "note", blurb: "Notion 風 WYSIWYG markdown 分頁。", note: "Milkdown 編輯器實例(架構規則 #1:handle 不進 $state)+ 需真實 markdown tab,不在 gallery 內掛載。" },
          { name: "PdfTab", importPath: "$lib/viewers/PdfTab.svelte", kind: "note", blurb: "PDF.js 虛擬化頁面預覽。", note: "需真實 PDF bytes(read_binary_file),不在 gallery 內掛載。" },
          { name: "TextTab", importPath: "$lib/viewers/TextTab.svelte", kind: "note", blurb: "CodeMirror 唯讀 + 語言偵測。", note: "需真實檔案內容 + CodeMirror 實例,不在 gallery 內掛載。" },
          { name: "ImageTab", importPath: "$lib/viewers/ImageTab.svelte", kind: "note", blurb: "圖片預覽分頁。", note: "需真實圖片路徑(asset 協定),不在 gallery 內掛載。" },
        ],
      },
    ],
  },

  // ── distill ────────────────────────────────────────────────────────────────
  {
    id: "distill",
    title: "distill",
    chip: "marrow://distill",
    blurb: "L0→L1 名詞提煉整頁,自帶獨立 explorer(不用全域 Sidebar)。",
    groups: [
      {
        label: "Distill",
        accent: "primary",
        items: [
          {
            name: "DistillExplorer",
            importPath: "$lib/distill/DistillExplorer.svelte",
            kind: "seed",
            blurb: "Distill 自帶的左欄選檔器。此處用 mock transport,列出 devmock 範例 .md。",
            examples: [
              {
                props: () => ({
                  root: workspace.info?.root ?? "",
                  transport: "mock",
                  selected: null,
                  onpick: noop,
                  onroot: noop,
                }),
              },
            ],
          },
          {
            name: "DistillTab",
            importPath: "$lib/distill/DistillTab.svelte",
            kind: "note",
            blurb: "整頁 Distill 視圖(explorer + 提煉結果)。",
            note: "瀏覽器下預設走 HTTP transport(:7080 bridge),且選檔後會呼叫 extractAnnotations;不在 gallery 內掛載。整頁效果見上方 route 說明。",
          },
        ],
      },
    ],
  },
];
