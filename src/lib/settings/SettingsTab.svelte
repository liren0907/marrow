<script lang="ts">
  // Settings rendered as a regular workspace tab (kind: "settings", virtual
  // path "marrow://settings"). Replaces the old SettingsModal — see
  // settingsModalState.svelte.ts for the open/close API which now operates
  // on the tab system instead of an isOpen flag.
  //
  // The section nav lives inside the tab body itself (NOT in the tab bar)
  // so the marrow tab bar stays uniform — every tab gets one entry, and
  // settings looks like any other view.
  import {
    settingsModal,
    setSettingsTab,
    type SettingsTab,
  } from "./settingsModalState.svelte";
  import Icon, { type IconName } from "$lib/components/ui/Icon.svelte";
  import AppearanceSection from "./sections/AppearanceSection.svelte";
  import EditorSection from "./sections/EditorSection.svelte";
  import KeyboardSection from "./sections/KeyboardSection.svelte";
  import WorkspaceSection from "./sections/WorkspaceSection.svelte";
  import AdvancedSection from "./sections/AdvancedSection.svelte";
  import AboutSection from "./sections/AboutSection.svelte";

  interface SectionSpec {
    id: SettingsTab;
    label: string;
    icon: IconName;
  }

  const sections: SectionSpec[] = [
    { id: "appearance", label: "Appearance", icon: "sliders-horizontal" },
    { id: "editor", label: "Editor", icon: "pencil" },
    { id: "keyboard", label: "Keyboard", icon: "keyboard" },
    { id: "workspace", label: "Workspace", icon: "folder" },
    { id: "advanced", label: "Advanced", icon: "flask-conical" },
    { id: "about", label: "About", icon: "info" },
  ];
</script>

<div class="settings-tab">
  <nav class="settings-nav" aria-label="Settings sections">
    {#each sections as section (section.id)}
      <button
        class="nav-item"
        class:active={settingsModal.activeTab === section.id}
        onclick={() => setSettingsTab(section.id)}
        aria-pressed={settingsModal.activeTab === section.id}
      >
        <Icon name={section.icon} size={14} />
        <span>{section.label}</span>
      </button>
    {/each}
  </nav>

  <section class="settings-content">
    {#if settingsModal.activeTab === "appearance"}
      <AppearanceSection />
    {:else if settingsModal.activeTab === "editor"}
      <EditorSection />
    {:else if settingsModal.activeTab === "keyboard"}
      <KeyboardSection />
    {:else if settingsModal.activeTab === "workspace"}
      <WorkspaceSection />
    {:else if settingsModal.activeTab === "advanced"}
      <AdvancedSection />
    {:else if settingsModal.activeTab === "about"}
      <AboutSection />
    {/if}
  </section>
</div>

<style>
  .settings-tab {
    height: 100%;
    display: grid;
    grid-template-columns: 200px 1fr;
    background: var(--color-base-100);
    color: var(--color-base-content);
    overflow: hidden;
  }
  .settings-nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 16px 10px;
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
    /* Generous padding mirrors the modal's old layout — long-form reading
       comfort matters since the user is scanning descriptions, not just
       toggling switches. */
    padding: 28px 36px 48px;
  }
</style>
