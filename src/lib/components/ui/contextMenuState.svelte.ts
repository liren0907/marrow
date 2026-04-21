import type { IconName } from "./Icon.svelte";

export interface ContextMenuItem {
  label: string;
  icon?: IconName;
  danger?: boolean;
  divider?: boolean;
  disabled?: boolean;
  onclick?: () => void | Promise<void>;
}

export const contextMenu = $state<{
  isOpen: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
}>({
  isOpen: false,
  x: 0,
  y: 0,
  items: [],
});

export function openContextMenu(e: MouseEvent, items: ContextMenuItem[]): void {
  e.preventDefault();
  e.stopPropagation();
  contextMenu.isOpen = true;
  contextMenu.x = e.clientX;
  contextMenu.y = e.clientY;
  contextMenu.items = items;
}

export function closeContextMenu(): void {
  contextMenu.isOpen = false;
  contextMenu.items = [];
}
