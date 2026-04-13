export const quickOpen = $state<{
  isOpen: boolean;
  query: string;
  selectedIdx: number;
}>({
  isOpen: false,
  query: "",
  selectedIdx: 0,
});
