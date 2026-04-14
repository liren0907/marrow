export const commandPalette = $state({
  isOpen: false,
  query: "",
  selectedIdx: 0,
});

export function toggleCommandPalette(): void {
  commandPalette.isOpen = !commandPalette.isOpen;
  if (commandPalette.isOpen) {
    commandPalette.query = "";
    commandPalette.selectedIdx = 0;
  }
}
