<script lang="ts">
  import {
    contextMenu,
    closeContextMenu,
    type ContextMenuItem,
  } from "./contextMenuState.svelte";

  let menuEl: HTMLDivElement | undefined = $state();
  let selectedIdx = $state(0);

  // Indexes of items that can actually receive focus (skip dividers / disabled).
  const focusable = $derived(
    contextMenu.items
      .map((item, i) => ({ item, i }))
      .filter(({ item }) => !item.divider && !item.disabled)
      .map(({ i }) => i),
  );

  // When the menu opens (or items change), reset selection to the first focusable.
  $effect(() => {
    if (contextMenu.isOpen) {
      selectedIdx = focusable[0] ?? 0;
      // Focus the menu container so keydown bubbles work even before user mouses in.
      queueMicrotask(() => menuEl?.focus());
    }
  });

  // Clamp the menu position to keep it on-screen.
  const position = $derived.by(() => {
    if (!contextMenu.isOpen) return { left: 0, top: 0 };
    if (typeof window === "undefined") {
      return { left: contextMenu.x, top: contextMenu.y };
    }
    const w = menuEl?.offsetWidth ?? 200;
    const h = menuEl?.offsetHeight ?? 8;
    const maxLeft = window.innerWidth - w - 8;
    const maxTop = window.innerHeight - h - 8;
    return {
      left: Math.min(Math.max(8, contextMenu.x), maxLeft),
      top: Math.min(Math.max(8, contextMenu.y), maxTop),
    };
  });

  function moveSelection(delta: number): void {
    if (focusable.length === 0) return;
    const cur = focusable.indexOf(selectedIdx);
    const next = cur < 0 ? 0 : (cur + delta + focusable.length) % focusable.length;
    selectedIdx = focusable[next];
  }

  function handleWindowKey(e: KeyboardEvent) {
    if (!contextMenu.isOpen) return;
    if (e.key === "Escape") {
      e.preventDefault();
      closeContextMenu();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      moveSelection(1);
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      moveSelection(-1);
      return;
    }
    if (e.key === "Home") {
      e.preventDefault();
      selectedIdx = focusable[0] ?? 0;
      return;
    }
    if (e.key === "End") {
      e.preventDefault();
      selectedIdx = focusable[focusable.length - 1] ?? 0;
      return;
    }
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      const item = contextMenu.items[selectedIdx];
      if (item) void handleItemClick(item);
      return;
    }
  }

  function handleWindowClick(e: MouseEvent) {
    if (!contextMenu.isOpen) return;
    if (menuEl && menuEl.contains(e.target as Node)) return;
    closeContextMenu();
  }

  function handleWindowBlur() {
    if (contextMenu.isOpen) closeContextMenu();
  }

  function handleScroll() {
    if (contextMenu.isOpen) closeContextMenu();
  }

  function handleWindowContextMenu(e: MouseEvent) {
    if (contextMenu.isOpen && menuEl && !menuEl.contains(e.target as Node)) {
      closeContextMenu();
    }
  }

  async function handleItemClick(item: ContextMenuItem) {
    if (item.disabled) return;
    closeContextMenu();
    if (item.onclick) {
      try {
        await item.onclick();
      } catch (e) {
        console.error("[ContextMenu] item click failed", e);
      }
    }
  }
</script>

<svelte:window
  onkeydown={handleWindowKey}
  onmousedown={handleWindowClick}
  onblur={handleWindowBlur}
  onscroll={handleScroll}
  oncontextmenu={handleWindowContextMenu}
/>

{#if contextMenu.isOpen}
  <div
    bind:this={menuEl}
    class="context-menu"
    style:left="{position.left}px"
    style:top="{position.top}px"
    role="menu"
    tabindex="-1"
  >
    {#each contextMenu.items as item, i (i)}
      {#if item.divider}
        <div class="divider" role="separator"></div>
      {:else}
        <button
          type="button"
          class="item"
          class:danger={item.danger}
          class:disabled={item.disabled}
          class:selected={i === selectedIdx}
          disabled={item.disabled}
          role="menuitem"
          onclick={() => handleItemClick(item)}
          onmouseenter={() => (selectedIdx = i)}
        >
          {#if item.icon}
            <span class="material-symbols-rounded text-[16px] shrink-0"
              >{item.icon}</span
            >
          {/if}
          <span class="label">{item.label}</span>
        </button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .context-menu {
    position: fixed;
    z-index: 100;
    min-width: 180px;
    padding: 0.25rem;
    background-color: oklch(var(--b1));
    border: 1px solid oklch(var(--b3));
    border-radius: 0.5rem;
    box-shadow: 0 8px 24px oklch(0 0 0 / 0.18);
    display: flex;
    flex-direction: column;
    gap: 0.0625rem;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.6rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: oklch(var(--bc));
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
  }
  .item:hover:not(.disabled),
  .item.selected:not(.disabled) {
    background-color: oklch(var(--b2));
  }
  .context-menu:focus {
    outline: none;
  }
  .item.danger {
    color: oklch(var(--er));
  }
  .item.disabled {
    color: oklch(var(--bc) / 0.4);
    cursor: not-allowed;
  }
  .divider {
    height: 1px;
    background-color: oklch(var(--b3));
    margin: 0.25rem 0;
  }
</style>
