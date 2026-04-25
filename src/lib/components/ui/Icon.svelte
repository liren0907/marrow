<script lang="ts" module>
  // Icon data comes from the vanilla `lucide` package (raw iconNode arrays).
  // We render the SVG ourselves here so there is no dependency on any
  // .svelte files in node_modules — avoids Svelte 5 template-TS parsing
  // issues that ship with @lucide/svelte. Named imports tree-shake because
  // the package declares `"sideEffects": false`.
  import {
    Plus,
    ArrowLeft,
    ChevronRight,
    ChevronDown,
    ChevronUp,
    X,
    FolderPlus,
    Trash2,
    FileText,
    File,
    FileCode,
    Pencil,
    Folder,
    FolderOpen,
    History,
    Network,
    Image,
    Film,
    Music,
    FilePlus,
    Maximize2,
    ExternalLink,
    Search,
    Tag,
    Terminal,
    Columns2,
    Info,
    CircleCheck,
    TriangleAlert,
    CircleAlert,
    RotateCcw,
    MoveRight,
    SlidersHorizontal,
    FlaskConical,
  } from "lucide";

  export type IconName =
    | "plus"
    | "arrow-left"
    | "chevron-right"
    | "chevron-down"
    | "chevron-up"
    | "x"
    | "folder-plus"
    | "trash-2"
    | "file-text"
    | "file"
    | "file-code"
    | "pencil"
    | "folder"
    | "folder-open"
    | "history"
    | "network"
    | "image"
    | "film"
    | "music"
    | "file-plus"
    | "maximize-2"
    | "external-link"
    | "search"
    | "tag"
    | "terminal"
    | "columns-2"
    | "info"
    | "circle-check"
    | "triangle-alert"
    | "circle-alert"
    | "rotate-ccw"
    | "move-right"
    | "sliders-horizontal"
    | "flask-conical";

  const MAP = {
    "plus": Plus,
    "arrow-left": ArrowLeft,
    "chevron-right": ChevronRight,
    "chevron-down": ChevronDown,
    "chevron-up": ChevronUp,
    "x": X,
    "folder-plus": FolderPlus,
    "trash-2": Trash2,
    "file-text": FileText,
    "file": File,
    "file-code": FileCode,
    "pencil": Pencil,
    "folder": Folder,
    "folder-open": FolderOpen,
    "history": History,
    "network": Network,
    "image": Image,
    "film": Film,
    "music": Music,
    "file-plus": FilePlus,
    "maximize-2": Maximize2,
    "external-link": ExternalLink,
    "search": Search,
    "tag": Tag,
    "terminal": Terminal,
    "columns-2": Columns2,
    "info": Info,
    "circle-check": CircleCheck,
    "triangle-alert": TriangleAlert,
    "circle-alert": CircleAlert,
    "rotate-ccw": RotateCcw,
    "move-right": MoveRight,
    "sliders-horizontal": SlidersHorizontal,
    "flask-conical": FlaskConical,
  } satisfies Record<IconName, unknown>;
</script>

<script lang="ts">
  interface Props {
    name: IconName;
    size?: number;
    strokeWidth?: number;
    class?: string;
  }
  let {
    name,
    size = 16,
    strokeWidth = 1.5,
    class: className = "",
  }: Props = $props();

  const nodes = $derived(MAP[name]);
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  class={`mw-icon ${className}`}
  aria-hidden="true"
>
  {#each nodes as [tag, attrs] (tag + JSON.stringify(attrs))}
    <svelte:element this={tag} {...attrs} />
  {/each}
</svg>

<style>
  :global(.mw-icon) {
    display: inline-block;
    vertical-align: middle;
    flex-shrink: 0;
  }
</style>
