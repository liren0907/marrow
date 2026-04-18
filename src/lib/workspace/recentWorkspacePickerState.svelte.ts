export const recentWorkspacePicker = $state({
  isOpen: false,
  query: "",
  selectedIdx: 0,
});

export function openRecentWorkspacePicker(): void {
  recentWorkspacePicker.isOpen = true;
  recentWorkspacePicker.query = "";
  recentWorkspacePicker.selectedIdx = 0;
}

export function closeRecentWorkspacePicker(): void {
  recentWorkspacePicker.isOpen = false;
}
