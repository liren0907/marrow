<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { tags, tagList, filesForTag } from "$lib/workspace/tagIndex.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

  let filter = $state("");
  let openTags = $state(new Set<string>());

  const allTags = $derived.by(() => {
    void tags.byTag.size;
    void tags.lastBuilt;
    return tagList();
  });

  const filtered = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return allTags;
    return allTags.filter((t) => t.tag.includes(q));
  });

  function toggleTag(tag: string): void {
    const next = new Set(openTags);
    if (next.has(tag)) next.delete(tag);
    else next.add(tag);
    openTags = next;
  }

  function relPath(path: string): string {
    const root = workspace.info?.root ?? "";
    if (root && path.startsWith(root)) {
      return path.slice(root.length).replace(/^[/\\]/, "");
    }
    return path;
  }

  function open(path: string): void {
    workspace.replaceCurrentTab(path);
  }
</script>

<div class="flex-1 flex flex-col min-h-0">
  <div class="px-2 py-1 border-b border-base-200 shrink-0">
    <input
      type="text"
      class="w-full bg-transparent outline-none text-xs px-2 py-1 placeholder:text-base-content/40"
      placeholder="Filter tags…"
      bind:value={filter}
    />
  </div>
  <div class="flex-1 overflow-y-auto py-2">
    {#if filtered.length === 0}
      <p class="px-3 py-2 text-xs text-base-content/40 italic">
        {allTags.length === 0
          ? "No tags found in workspace"
          : "No tags match filter"}
      </p>
    {:else}
      <ul class="flex flex-col gap-0.5">
        {#each filtered as { tag, count } (tag)}
          <li>
            <button
              type="button"
              class="sidebar-row gap-2"
              onclick={() => toggleTag(tag)}
            >
              <span class="sidebar-chev-slot">
                <Icon
                  name={openTags.has(tag) ? "chevron-down" : "chevron-right"}
                  size={14}
                />
              </span>
              <!-- `#tag` is a token, not natural language — the mono +
                   primary-accent treatment makes it visually distinct from
                   FileTree's plain-prose filenames. Keep this even though
                   .sidebar-row standardizes everything else; the visual
                   distinction is intentional, not an oversight. -->
              <span class="font-mono text-primary truncate flex-1">#{tag}</span>
              <span class="text-[11px] text-base-content/40">{count}</span>
            </button>
            {#if openTags.has(tag)}
              <ul class="mt-0.5 mb-1 flex flex-col gap-0.5">
                {#each filesForTag(tag) as path (path)}
                  <li>
                    <button
                      type="button"
                      class="sidebar-row gap-2"
                      onclick={() => open(path)}
                    >
                      <!-- chev-spacer keeps the file-text icon aligned under the
                           parent tag's text (not under its chevron). -->
                      <span class="sidebar-chev-spacer" aria-hidden="true"></span>
                      <Icon name="file-text" size={12} class="text-base-content/40" />
                      <span class="truncate">{relPath(path)}</span>
                    </button>
                  </li>
                {/each}
              </ul>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>
