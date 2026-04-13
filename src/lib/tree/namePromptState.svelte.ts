export const namePrompt = $state<{
  isOpen: boolean;
  title: string;
  placeholder: string;
  initial: string;
  confirmLabel: string;
  onConfirm: ((value: string) => void | Promise<void>) | null;
}>({
  isOpen: false,
  title: "",
  placeholder: "",
  initial: "",
  confirmLabel: "Create",
  onConfirm: null,
});

export function openNamePrompt(opts: {
  title: string;
  placeholder?: string;
  initial?: string;
  confirmLabel?: string;
  onConfirm: (value: string) => void | Promise<void>;
}): void {
  namePrompt.isOpen = true;
  namePrompt.title = opts.title;
  namePrompt.placeholder = opts.placeholder ?? "";
  namePrompt.initial = opts.initial ?? "";
  namePrompt.confirmLabel = opts.confirmLabel ?? "Create";
  namePrompt.onConfirm = opts.onConfirm;
}

export function closeNamePrompt(): void {
  namePrompt.isOpen = false;
  namePrompt.onConfirm = null;
}
