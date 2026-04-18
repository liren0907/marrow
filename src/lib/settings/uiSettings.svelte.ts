const STORAGE_KEY = "marrow.ui";

interface Persisted {
  showBreadcrumb: boolean;
  showPaneOutline: boolean;
}

const DEFAULTS: Persisted = {
  showBreadcrumb: true,
  showPaneOutline: true,
};

function loadPersisted(): Persisted {
  if (typeof localStorage === "undefined") return { ...DEFAULTS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as Partial<Persisted>;
    return { ...DEFAULTS, ...parsed };
  } catch {
    return { ...DEFAULTS };
  }
}

export const uiSettings = $state<Persisted>(loadPersisted());

export function persistUiSettings(): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        showBreadcrumb: uiSettings.showBreadcrumb,
        showPaneOutline: uiSettings.showPaneOutline,
      }),
    );
  } catch {
    // ignore
  }
}

export function toggleBreadcrumb(): void {
  uiSettings.showBreadcrumb = !uiSettings.showBreadcrumb;
  persistUiSettings();
}

export function togglePaneOutline(): void {
  uiSettings.showPaneOutline = !uiSettings.showPaneOutline;
  persistUiSettings();
}
