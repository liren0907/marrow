import type { RefactorPreview } from "./renameRefactor";

export const renameModal = $state<{
  isOpen: boolean;
  oldPath: string;
  stage: "name" | "preview" | "running";
  newName: string;
  preview: RefactorPreview | null;
}>({
  isOpen: false,
  oldPath: "",
  stage: "name",
  newName: "",
  preview: null,
});

export function startFileRename(path: string): void {
  const name = path.split(/[/\\]/).pop() ?? path;
  renameModal.isOpen = true;
  renameModal.oldPath = path;
  renameModal.stage = "name";
  renameModal.newName = name;
  renameModal.preview = null;
}

export function closeRenameModal(): void {
  renameModal.isOpen = false;
  renameModal.oldPath = "";
  renameModal.stage = "name";
  renameModal.newName = "";
  renameModal.preview = null;
}
