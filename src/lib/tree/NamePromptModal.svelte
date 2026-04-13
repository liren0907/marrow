<script lang="ts">
  import SimpleModal from "$lib/components/ui/SimpleModal.svelte";
  import { namePrompt, closeNamePrompt } from "./namePromptState.svelte";

  let value = $state("");
  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (namePrompt.isOpen) {
      value = namePrompt.initial;
      queueMicrotask(() => {
        inputEl?.focus();
        inputEl?.select();
      });
    }
  });

  async function confirm() {
    const fn = namePrompt.onConfirm;
    const v = value;
    closeNamePrompt();
    if (fn && v.trim()) {
      await fn(v.trim());
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      void confirm();
    }
  }
</script>

<SimpleModal
  isOpen={namePrompt.isOpen}
  title={namePrompt.title}
  onclose={closeNamePrompt}
>
  {#snippet children()}
    <input
      bind:this={inputEl}
      type="text"
      class="input input-bordered w-full"
      placeholder={namePrompt.placeholder}
      bind:value
      onkeydown={onKey}
    />
  {/snippet}
  {#snippet actions()}
    <button type="button" class="btn btn-ghost btn-sm" onclick={closeNamePrompt}>
      Cancel
    </button>
    <button type="button" class="btn btn-primary btn-sm" onclick={confirm}>
      {namePrompt.confirmLabel}
    </button>
  {/snippet}
</SimpleModal>
