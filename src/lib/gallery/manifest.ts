import type { IconName } from "$lib/components/ui/Icon.svelte";
import { workspace } from "$lib/workspace/workspace.svelte";
import { pathForImport } from "./discovery";
import {
  imageTab,
  videoTab,
  audioTab,
  unsupportedTab,
  textTab,
  seededMarkdownTab,
} from "./sampleTabs";

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
  // Full-page components (height:100%) need a bounded height to render — when
  // set, the preview renders inside a framed box of this many pixels tall.
  previewHeight?: number;
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
    ],
  },

  // ── viewers (file-content viewers, dispatched by TabBody on tab.kind) ────────
  {
    id: "viewers",
    title: "viewers",
    chip: "$lib/viewers",
    blurb:
      "開檔時負責渲染檔案內容的元件(TabBody 依 tab.kind 分派)。gallery 端用假 tab / 範例素材離線真渲染,不發任何 Tauri、不改 viewer 本身。MarkdownTab 重用 seed 開好的真 tab。",
    groups: [
      {
        label: "Text & code",
        accent: "primary",
        items: [
          {
            name: "MarkdownTab",
            importPath: "$lib/viewers/MarkdownTab.svelte",
            kind: "seed",
            blurb: "Notion 風 WYSIWYG markdown 分頁(Milkdown)。此處重用 seed 開好的真 markdown tab 渲染。",
            previewHeight: 460,
            examples: [{ props: () => ({ tab: seededMarkdownTab() }) }],
          },
          {
            name: "TextTab",
            importPath: "$lib/viewers/TextTab.svelte",
            kind: "seed",
            blurb: "CodeMirror 唯讀 + 語言偵測。此處餵一個 mock .md 當來源,顯示其原始碼。",
            previewHeight: 320,
            examples: [{ props: () => ({ tab: textTab() }) }],
          },
        ],
      },
      {
        label: "Media",
        accent: "accent",
        items: [
          {
            name: "ImageTab",
            importPath: "$lib/viewers/ImageTab.svelte",
            kind: "props",
            blurb: "圖片預覽分頁。此處用內嵌 SVG 範例圖。",
            previewHeight: 200,
            examples: [{ props: () => ({ tab: imageTab() }) }],
          },
          {
            name: "VideoTab",
            importPath: "$lib/viewers/VideoTab.svelte",
            kind: "props",
            blurb: "影片預覽分頁(原生 <video> controls)。此處用幾 KB 範例 mp4。",
            previewHeight: 200,
            examples: [{ props: () => ({ tab: videoTab() }) }],
          },
          {
            name: "AudioTab",
            importPath: "$lib/viewers/AudioTab.svelte",
            kind: "props",
            blurb: "音訊預覽分頁(原生 <audio> controls)。此處用範例 wav。",
            previewHeight: 200,
            examples: [{ props: () => ({ tab: audioTab() }) }],
          },
        ],
      },
      {
        label: "Document",
        accent: "secondary",
        items: [
          {
            name: "UnsupportedTab",
            importPath: "$lib/viewers/UnsupportedTab.svelte",
            kind: "props",
            blurb: "無對應 viewer 的檔案 fallback(icon + 標題 +「以系統開啟」)。",
            previewHeight: 200,
            examples: [{ props: () => ({ tab: unsupportedTab() }) }],
          },
          {
            name: "PdfTab",
            importPath: "$lib/viewers/PdfTab.svelte",
            kind: "note",
            blurb: "PDF.js 虛擬化頁面預覽。",
            note: "需真實 PDF bytes(走 read_binary_file 讀取路徑),gallery 端餵不進去,維持不掛載。",
          },
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
            name: "DistillPanel",
            importPath: "$lib/distill/DistillPanel.svelte",
            kind: "props",
            blurb:
              "提煉結果面板(DistillTab 的右半,已抽成獨立元件)。純展示:依 status 顯示 5 種畫面,ready 時列出 詞 + 詞性 Badge + 次數。",
            previewHeight: 104,
            examples: [
              {
                label: "ready",
                props: {
                  status: "ready",
                  annotations: [
                    { text: "範例名詞", pos: "n", count: 3 },
                    { text: "知識圖譜", pos: "n", count: 2 },
                    { text: "marrow", pos: "eng", count: 1 },
                  ],
                },
              },
              { label: "empty", props: { status: "empty" } },
              { label: "loading", props: { status: "loading" } },
              { label: "error", props: { status: "error", errorMessage: "讀取失敗：ENOENT" } },
              { label: "no terms", props: { status: "ready", annotations: [] } },
            ],
          },
          {
            name: "DistillTab",
            importPath: "$lib/distill/DistillTab.svelte",
            kind: "seed",
            blurb:
              "整頁 Distill 視圖(左 explorer + 右名詞提煉)。此處以 preview 模式用 mock transport 離線真渲染,預選一篇 .md 顯示提煉結果(devmock 範例詞),不讀寫真實 Distill 狀態。",
            previewHeight: 440,
            examples: [
              {
                props: () => {
                  const root = workspace.info?.root ?? "";
                  const md = workspace.fileIndex.find((f) => f.kind === "markdown");
                  return { transport: "mock", initialRoot: root, initialSource: md?.path };
                },
              },
            ],
          },
        ],
      },
    ],
  },
];
