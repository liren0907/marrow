<script lang="ts">
  import { fade, scale } from "svelte/transition";
  import {
    settingsModal,
    closeSettings,
    setSettingsTab,
    type SettingsTab,
  } from "./settingsModalState.svelte";
  import Icon, { type IconName } from "$lib/components/ui/Icon.svelte";
  import AppearanceSection from "./sections/AppearanceSection.svelte";
  import EditorSection from "./sections/EditorSection.svelte";
  import WorkspaceSection from "./sections/WorkspaceSection.svelte";
  import AdvancedSection from "./sections/AdvancedSection.svelte";
  import AboutSection from "./sections/AboutSection.svelte";

  interface TabSpec {
    id: SettingsTab;
    label: string;
    icon: IconName;
  }

  const tabs: TabSpec[] = [
    { id: "appearance", label: "Appearance", icon: "sliders-horizontal" },
    { id: "editor", label: "Editor", icon: "pencil" },
    { id: "workspace", label: "Workspace", icon: "folder" },
    { id: "advanced", label: "Advanced", icon: "flask-conical" },
    { id: "about", label: "About", icon: "info" },
  ];

  function onBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) closeSettings();
  }

  function onKeydown(e: KeyboardEvent) {
    if (!settingsModal.isOpen) return;
    if (e.key === "Escape") {
      e.preventDefault();
      closeSettings();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if settingsModal.isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    class="settings-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Settings"
    onclick={onBackdropClick}
    transition:fade={{ duration: 120 }}
  >
    <div
      class="settings-shell"
      transition:scale={{ start: 0.97, duration: 160 }}
      role="document"
    >
      <header class="settings-head">
        <h2 class="settings-title">Settings</h2>
        <button
          class="close-btn"
          onclick={closeSettings}
          aria-label="Close settings"
        >
          <Icon name="x" size={16} />
        </button>
      </header>

      <div class="settings-body">
        <nav class="settings-nav" aria-label="Settings sections">
          {#each tabs as tab (tab.id)}
            <button
              class="nav-item"
              class:active={settingsModal.activeTab === tab.id}
              onclick={() => setSettingsTab(tab.id)}
              aria-pressed={settingsModal.activeTab === tab.id}
            >
              <Icon name={tab.icon} size={14} />
              <span>{tab.label}</span>
            </button>
          {/each}
        </nav>

        <section class="settings-content">
          {#if settingsModal.activeTab === "appearance"}
            <AppearanceSection />
          {:else if settingsModal.activeTab === "editor"}
            <EditorSection />
          {:else if settingsModal.activeTab === "workspace"}
            <WorkspaceSection />
          {:else if settingsModal.activeTab === "advanced"}
            <AdvancedSection />
          {:else if settingsModal.activeTab === "about"}
            <AboutSection />
          {/if}
        </section>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-backdrop {
    position: fixed;
    inset: 0;
    background: oklch(0 0 0 / 0.5);
    display: grid;
    place-items: center;
    z-index: 70;
    backdrop-filter: blur(2px);
  }
  .settings-shell {
    width: min(820px, 92vw);
    height: min(560px, 86vh);
    background: var(--mw-bg-elev, var(--color-base-100));
    border: 1px solid var(--mw-rule-strong, var(--mw-rule));
    border-radius: var(--mw-radius-md);
    box-shadow: 0 24px 56px oklch(0 0 0 / 0.3);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    color: var(--color-base-content);
  }
  .settings-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--mw-rule);
    background: var(--color-base-100);
  }
  .settings-title {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
  }
  .close-btn {
    background: transparent;
    border: none;
    padding: 4px;
    border-radius: var(--mw-radius-xs);
    color: var(--mw-ink-2);
    cursor: pointer;
    display: grid;
    place-items: center;
  }
  .close-btn:hover {
    background: var(--color-base-300);
    color: var(--color-base-content);
  }
  .settings-body {
    display: grid;
    grid-template-columns: 180px 1fr;
    flex: 1;
    min-height: 0;
  }
  .settings-nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px 8px;
    background: var(--color-base-200);
    border-right: 1px solid var(--mw-rule);
    overflow-y: auto;
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    border: none;
    background: transparent;
    color: var(--mw-ink-2);
    font-size: 12.5px;
    text-align: left;
    border-radius: var(--mw-radius-xs);
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }
  .nav-item:hover {
    background: var(--color-base-300);
    color: var(--color-base-content);
  }
  .nav-item.active {
    background: var(--color-base-100);
    color: var(--color-base-content);
    box-shadow: inset 2px 0 0 var(--mw-accent);
  }
  .settings-content {
    overflow-y: auto;
    padding: 24px 28px;
  }
</style>
