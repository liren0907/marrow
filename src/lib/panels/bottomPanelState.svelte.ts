export type ActiveTab =
  | "backlinks"
  | "unresolved"
  | "tags"
  | "outline"
  | "peek";

const STORAGE_KEY = "marrow.bottomPanel";

interface Persisted {
  isOpen: boolean;
  height: number;
  activeTab: ActiveTab;
}

function loadPersisted(): Persisted {
  if (typeof localStorage === "undefined") {
    return { isOpen: false, height: 220, activeTab: "backlinks" };
  }
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { isOpen: false, height: 220, activeTab: "backlinks" };
    const parsed = JSON.parse(raw) as Partial<Persisted>;
    let activeTab: ActiveTab = "backlinks";
    if (parsed.activeTab === "unresolved") activeTab = "unresolved";
    else if (parsed.activeTab === "tags") activeTab = "tags";
    else if (parsed.activeTab === "outline") activeTab = "outline";
    // Peek is session-only — don't restore it (peek.layers is empty on load).
    // Fallback to backlinks instead.
    return {
      isOpen: !!parsed.isOpen,
      height: typeof parsed.height === "number" ? parsed.height : 220,
      activeTab,
    };
  } catch {
    return { isOpen: false, height: 220, activeTab: "backlinks" };
  }
}

export const bottomPanel = $state<Persisted>(loadPersisted());

export function persistBottomPanel(): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        isOpen: bottomPanel.isOpen,
        height: bottomPanel.height,
        activeTab: bottomPanel.activeTab,
      }),
    );
  } catch {
    // ignore
  }
}

export function toggleBottomPanel(): void {
  bottomPanel.isOpen = !bottomPanel.isOpen;
  persistBottomPanel();
}
