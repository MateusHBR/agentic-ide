<script lang="ts">
  import { onMount } from "svelte";
  import { appState } from "$lib/state.svelte";
  import type { ProjectInfo, WorktreeInfo } from "$lib/state.svelte";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();

  interface FlatItem {
    project: ProjectInfo;
    worktree: WorktreeInfo;
    label: string;
    sublabel: string;
  }

  let searchQuery = $state("");
  let inputEl = $state<HTMLInputElement | null>(null);
  let listEl = $state<HTMLDivElement | null>(null);

  // Flatten all worktrees into a single list
  let allItems = $derived.by(() => {
    const items: FlatItem[] = [];
    for (const project of appState.projects) {
      for (const wt of project.worktrees) {
        items.push({
          project,
          worktree: wt,
          label: `${project.name} — ${wt.branch}`,
          sublabel: wt.path,
        });
      }
    }
    return items;
  });

  function getInitialIndex(): number {
    const items: FlatItem[] = [];
    for (const project of appState.projects) {
      for (const wt of project.worktrees) {
        items.push({ project, worktree: wt, label: "", sublabel: "" });
      }
    }
    const idx = items.findIndex((item) => item.worktree.path === appState.activeWorktree);
    return idx >= 0 ? idx : 0;
  }

  let selectedIndex = $state(getInitialIndex());

  let filteredItems = $derived.by(() => {
    if (!searchQuery.trim()) return allItems;
    const q = searchQuery.toLowerCase();
    return allItems.filter(
      (item) =>
        item.label.toLowerCase().includes(q) ||
        item.sublabel.toLowerCase().includes(q)
    );
  });

  // Clamp selected index when search filters the list
  $effect(() => {
    const len = filteredItems.length;
    if (selectedIndex >= len) {
      selectedIndex = Math.max(0, len - 1);
    }
  });

  // Scroll to initial selection on mount
  onMount(() => {
    requestAnimationFrame(() => {
      const item = listEl?.children[selectedIndex] as HTMLElement | undefined;
      item?.scrollIntoView({ block: "nearest" });
    });
  });

  $effect(() => {
    if (inputEl) {
      inputEl.focus();
    }
  });

  function scrollToSelected() {
    requestAnimationFrame(() => {
      const item = listEl?.children[selectedIndex] as HTMLElement | undefined;
      item?.scrollIntoView({ block: "nearest" });
    });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filteredItems.length - 1);
      scrollToSelected();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      scrollToSelected();
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (filteredItems.length > 0) {
        selectItem(filteredItems[selectedIndex]);
      }
    }
  }

  function selectItem(item: FlatItem) {
    appState.selectWorktree(item.project.path, item.worktree.path);
    onClose();
  }

  function isActive(item: FlatItem): boolean {
    return appState.activeWorktree === item.worktree.path;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="overlay" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="switcher" onclick={(e) => e.stopPropagation()} onkeydown={handleKeydown}>
    <div class="search-bar">
      <span class="search-icon">⌘W</span>
      <input
        bind:this={inputEl}
        bind:value={searchQuery}
        class="search-input"
        type="text"
        placeholder="Switch worktree..."
      />
    </div>
    <div class="item-list" bind:this={listEl}>
      {#if filteredItems.length === 0}
        <div class="empty">No worktrees found</div>
      {:else}
        {#each filteredItems as item, i}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="item"
            class:selected={i === selectedIndex}
            class:current={isActive(item)}
            onclick={() => selectItem(item)}
            onmouseenter={() => (selectedIndex = i)}
          >
            <div class="item-left">
              <span class="item-dot" class:active={isActive(item)}></span>
              <div class="item-info">
                <div class="item-label">
                  <span class="item-project">{item.project.name}</span>
                  <span class="item-sep">—</span>
                  <span class="item-branch">{item.worktree.branch}</span>
                </div>
                <div class="item-path">{item.sublabel}</div>
              </div>
            </div>
            {#if isActive(item)}
              <span class="current-badge">current</span>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
    <div class="footer">
      <span class="hint"><kbd>↑↓</kbd> navigate</span>
      <span class="hint"><kbd>↵</kbd> select</span>
      <span class="hint"><kbd>esc</kbd> close</span>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 80px;
  }

  .switcher {
    width: 520px;
    max-height: 420px;
    background: #1c1c1e;
    border: 1px solid #3a3a3c;
    border-radius: 12px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    border-bottom: 1px solid #2a2a2c;
  }

  .search-icon {
    font-size: 12px;
    color: #636366;
    background: #2a2a2c;
    padding: 2px 6px;
    border-radius: 4px;
    flex-shrink: 0;
    font-family: 'SF Mono', Menlo, monospace;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    color: #e6edf3;
    font-size: 14px;
    font-family: inherit;
    -webkit-user-select: text;
    user-select: text;
  }

  .search-input::placeholder {
    color: #636366;
  }

  .item-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    cursor: pointer;
    transition: background 0.1s;
  }

  .item.selected {
    background: #2a2a2c;
  }

  .item.selected.current {
    background: #122b17;
  }

  .item-left {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .item-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #3a3a3c;
    flex-shrink: 0;
  }

  .item-dot.active {
    background: #30d158;
    box-shadow: 0 0 6px rgba(48, 209, 88, 0.5);
  }

  .item-info {
    min-width: 0;
  }

  .item-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
  }

  .item-project {
    color: #e6edf3;
    font-weight: 600;
  }

  .item-sep {
    color: #484f58;
  }

  .item-branch {
    color: #58a6ff;
  }

  .item.selected .item-branch {
    color: #79c0ff;
  }

  .item-path {
    font-size: 11px;
    color: #636366;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .current-badge {
    font-size: 10px;
    color: #30d158;
    background: rgba(48, 209, 88, 0.15);
    padding: 2px 8px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .empty {
    padding: 24px 16px;
    text-align: center;
    color: #636366;
    font-size: 13px;
  }

  .footer {
    display: flex;
    gap: 16px;
    padding: 8px 16px;
    border-top: 1px solid #2a2a2c;
  }

  .hint {
    font-size: 11px;
    color: #636366;
  }

  .hint kbd {
    font-family: 'SF Mono', Menlo, monospace;
    font-size: 10px;
    color: #8b949e;
    background: #2a2a2c;
    padding: 1px 4px;
    border-radius: 3px;
    margin-right: 3px;
  }
</style>
