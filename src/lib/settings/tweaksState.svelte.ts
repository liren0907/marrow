export const tweaks = $state<{ isOpen: boolean }>({ isOpen: false });

export function openTweaks(): void {
  tweaks.isOpen = true;
}

export function closeTweaks(): void {
  tweaks.isOpen = false;
}

export function toggleTweaks(): void {
  tweaks.isOpen = !tweaks.isOpen;
}
