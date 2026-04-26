// Workspace-scoped knobs (currently just the attachment folder used
// for pasted images). Persisted globally — a setting that depended on
// the open workspace would key off workspace.info.id instead.

const STORAGE_KEY = "marrow.workspace";

interface Persisted {
  /** Folder under workspace root where pasted images are written. */
  attachmentFolder: string;
}

const DEFAULTS: Persisted = {
  attachmentFolder: "attachments",
};

// Allow letters/digits/underscore/dash/dot, slash for nested paths.
// Reject leading slash, '..', and empty segments.
const FOLDER_RE = /^[A-Za-z0-9._-]+(\/[A-Za-z0-9._-]+)*$/;

export function isValidFolder(name: string): boolean {
  if (!name || name.length > 200) return false;
  if (name.startsWith("/") || name.startsWith("\\")) return false;
  if (name.split(/[/\\]/).some((seg) => seg === "" || seg === "..")) {
    return false;
  }
  return FOLDER_RE.test(name);
}

function loadPersisted(): Persisted {
  if (typeof localStorage === "undefined") return { ...DEFAULTS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as Partial<Persisted>;
    const folder = parsed.attachmentFolder;
    return {
      attachmentFolder:
        typeof folder === "string" && isValidFolder(folder)
          ? folder
          : DEFAULTS.attachmentFolder,
    };
  } catch {
    return { ...DEFAULTS };
  }
}

export const workspaceSettings = $state<Persisted>(loadPersisted());

function persist(): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        attachmentFolder: workspaceSettings.attachmentFolder,
      }),
    );
  } catch {
    // ignore
  }
}

/** Returns true if the value was accepted; false on validation failure. */
export function setAttachmentFolder(folder: string): boolean {
  const trimmed = folder.trim();
  if (!isValidFolder(trimmed)) return false;
  workspaceSettings.attachmentFolder = trimmed;
  persist();
  return true;
}

export function resetWorkspaceSettings(): void {
  workspaceSettings.attachmentFolder = DEFAULTS.attachmentFolder;
  persist();
}

export const WORKSPACE_DEFAULTS = DEFAULTS;
