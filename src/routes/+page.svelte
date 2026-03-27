<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import TerminalPanel from "$lib/components/TerminalPanel.svelte";
  import RightPanel from "$lib/components/RightPanel.svelte";
  import WorktreeSwitcher from "$lib/components/WorktreeSwitcher.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import ProfileManager from "$lib/components/ProfileManager.svelte";
  import UpdateToast from "$lib/components/UpdateToast.svelte";
  import { appState } from "$lib/state.svelte";
  import { profileState } from "$lib/profiles.svelte";

  let isResizingSidebar = $state(false);
  let isResizingRight = $state(false);
  let rightPanelWidth = $state(340);
  let showWorktreeSwitcher = $state(false);
  let showSettings = $state(false);
  let showProfiles = $state(false);
  let unlistenExit: UnlistenFn | null = null;

  function handleKeydown(e: KeyboardEvent) {
    if (e.metaKey && e.key === "p") {
      e.preventDefault();
      showProfiles = !showProfiles;
      showSettings = false;
      showWorktreeSwitcher = false;
      return;
    }
    if (e.metaKey && e.key === "w") {
      e.preventDefault();
      showWorktreeSwitcher = !showWorktreeSwitcher;
      showSettings = false;
      showProfiles = false;
      return;
    }
    if (e.metaKey && e.key === ",") {
      e.preventDefault();
      showSettings = !showSettings;
      showWorktreeSwitcher = false;
      showProfiles = false;
      return;
    }
    if (e.metaKey && e.key === "b") {
      e.preventDefault();
      appState.sidebarCollapsed = !appState.sidebarCollapsed;
      return;
    }
    if (e.metaKey && e.key === "j") {
      e.preventDefault();
      appState.rightPanelCollapsed = !appState.rightPanelCollapsed;
      return;
    }
    if (showWorktreeSwitcher || showSettings || showProfiles) return;
    if (e.metaKey && e.key === "n") {
      e.preventDefault();
      if (appState.activeWorktree) {
        handleNewTerminal(appState.activeWorktree);
      }
    }
    if (e.metaKey && e.key >= "1" && e.key <= "9") {
      e.preventDefault();
      const idx = parseInt(e.key) - 1;
      const worktreeTerminals = appState.activeWorktree
        ? appState.getTerminalsForWorktree(appState.activeWorktree)
        : [];
      if (idx < worktreeTerminals.length) {
        appState.setActiveTerminal(worktreeTerminals[idx].id);
      }
    }
  }

  function syncTrayProfiles() {
    invoke("sync_tray_profiles").catch(console.error);
  }

  onMount(async () => {
    // Determine which profile this window should use
    const urlParams = new URLSearchParams(window.location.search);
    const profileIdParam = urlParams.get("profile") ?? undefined;
    await appState.initializeWithProfile(profileIdParam);

    await appState.loadProjects();
    syncTrayProfiles();

    unlistenExit = await listen("terminal-exit", (event: any) => {
      const { id } = event.payload;
      invoke("close_terminal", { id }).catch(() => {});
      appState.removeTerminal(id);
    });

    window.addEventListener("keydown", handleKeydown, true);
    appState.startPolling();
  });

  onDestroy(() => {
    unlistenExit?.();
    window.removeEventListener("keydown", handleKeydown, true);
    appState.stopPolling();
  });

  async function handleNewTerminal(worktreePath: string) {
    try {
      const terminalId: string = await invoke("create_terminal", {
        cwd: worktreePath,
        cmd: null,
      });

      const worktreeTerminals = appState.getTerminalsForWorktree(worktreePath);
      appState.addTerminal({
        id: terminalId,
        name: `Terminal ${worktreeTerminals.length + 1}`,
        worktreePath,
      });
      appState.selectWorktree(appState.activeProject, worktreePath);
    } catch (e) {
      console.error("Failed to create terminal:", e);
    }
  }

  async function handleAddProject() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select a Git project folder",
      });
      if (selected) {
        const info = await appState.addProject(selected);
        if (info && info.worktrees.length > 0) {
          const mainWt = info.worktrees.find((w: any) => w.is_main) || info.worktrees[0];
          if (mainWt) {
            await handleNewTerminal(mainWt.path);
          }
        }
      }
    } catch (e) {
      console.error("Failed to add project:", e);
    }
  }

  function startResizeSidebar(e: MouseEvent) {
    isResizingSidebar = true;
    e.preventDefault();
    const onMove = (ev: MouseEvent) => {
      appState.sidebarWidth = Math.max(180, Math.min(window.innerWidth * 0.5, ev.clientX));
    };
    const onUp = () => {
      isResizingSidebar = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  // --- Tab rename ---
  let editingTabId = $state<string | null>(null);
  let editTabValue = $state("");
  let editTabInput = $state<HTMLInputElement | null>(null);

  function startTabRename(id: string, name: string) {
    editingTabId = id;
    editTabValue = name;
    requestAnimationFrame(() => editTabInput?.focus());
  }

  function commitTabRename() {
    if (editingTabId && editTabValue.trim()) {
      appState.renameTerminal(editingTabId, editTabValue.trim());
    }
    editingTabId = null;
  }

  function cancelTabRename() {
    editingTabId = null;
  }

  // --- Tab drag reorder ---
  let dragTabId = $state<string | null>(null);
  let dragOverTabId = $state<string | null>(null);

  function onTabDragStart(e: DragEvent, id: string) {
    dragTabId = id;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", id);
    }
  }

  function onTabDragOver(e: DragEvent, id: string) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragOverTabId = id;
  }

  function onTabDrop(e: DragEvent, targetId: string) {
    e.preventDefault();
    dragOverTabId = null;
    if (!dragTabId || dragTabId === targetId) { dragTabId = null; return; }
    const from = appState.terminals.findIndex((t) => t.id === dragTabId);
    const to = appState.terminals.findIndex((t) => t.id === targetId);
    if (from >= 0 && to >= 0) appState.reorderTerminals(from, to);
    dragTabId = null;
  }

  function onTabDragEnd() {
    dragTabId = null;
    dragOverTabId = null;
  }

  function startResizeRight(e: MouseEvent) {
    isResizingRight = true;
    e.preventDefault();
    const onMove = (ev: MouseEvent) => {
      if (appState.layout === "horizontal") {
        rightPanelWidth = Math.max(120, Math.min(window.innerHeight * 0.6, window.innerHeight - ev.clientY));
      } else {
        rightPanelWidth = Math.max(180, Math.min(window.innerWidth * 0.5, window.innerWidth - ev.clientX));
      }
    };
    const onUp = () => {
      isResizingRight = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }
</script>

<UpdateToast />

{#if showWorktreeSwitcher}
  <WorktreeSwitcher onClose={() => (showWorktreeSwitcher = false)} />
{/if}

{#if showSettings}
  <Settings onClose={() => (showSettings = false)} />
{/if}

{#if showProfiles}
  <ProfileManager onClose={() => { showProfiles = false; syncTrayProfiles(); }} />
{/if}

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="titlebar"
  data-tauri-drag-region
  style="border-bottom: 2px solid {profileState.activeProfile?.color ?? 'transparent'}"
  onmousedown={() => getCurrentWindow().startDragging()}
></div>

{#snippet terminalContent()}
  {#if appState.activeTerminalId && appState.activeWorktree && appState.getTerminalsForWorktree(appState.activeWorktree).length > 0}
    {#each appState.terminals as term (term.id)}
      <div
        class="terminal-tab"
        class:visible={term.id === appState.activeTerminalId}
      >
        <TerminalPanel terminalId={term.id} worktreePath={term.worktreePath} />
      </div>
    {/each}
  {:else}
    <div class="empty-terminal">
      <div class="empty-content">
        <div class="empty-icon">⬡</div>
        <h2>No Terminal Open</h2>
        <p>Click "New Terminal" on a worktree in the sidebar, or press <kbd>&#8984;N</kbd></p>
      </div>
    </div>
  {/if}

  {#if appState.activeWorktree && appState.getTerminalsForWorktree(appState.activeWorktree).length > 0}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="terminal-tabs-bar" onwheel={(e) => { e.preventDefault(); e.currentTarget.scrollLeft += e.deltaY; }}>
      {#each appState.getTerminalsForWorktree(appState.activeWorktree) as term, tabIdx (term.id)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="terminal-tab-btn"
          class:active={term.id === appState.activeTerminalId}
          class:tab-drag-over={dragOverTabId === term.id}
          class:tab-dragging={dragTabId === term.id}
          draggable="true"
          onclick={() => appState.setActiveTerminal(term.id)}
          ondblclick={() => startTabRename(term.id, term.name)}
          ondragstart={(e) => onTabDragStart(e, term.id)}
          ondragover={(e) => onTabDragOver(e, term.id)}
          ondragleave={() => (dragOverTabId = null)}
          ondrop={(e) => onTabDrop(e, term.id)}
          ondragend={onTabDragEnd}
        >
          {#if editingTabId === term.id}
            <input
              class="tab-rename-input"
              type="text"
              bind:this={editTabInput}
              bind:value={editTabValue}
              onblur={commitTabRename}
              onclick={(e) => e.stopPropagation()}
              onkeydown={(e) => {
                if (e.key === "Enter") commitTabRename();
                if (e.key === "Escape") cancelTabRename();
              }}
            />
          {:else}
            {#if tabIdx < 9}<span class="tab-shortcut">⌘{tabIdx + 1}</span>{/if}
            {term.name}
          {/if}
        </div>
      {/each}
    </div>
  {/if}
{/snippet}

{#if appState.layout === "vertical"}
  <div
    class="app-layout vertical"
    class:resizing={isResizingSidebar || isResizingRight}
  >
    {#if appState.sidebarCollapsed}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="collapsed-sidebar" onclick={() => (appState.sidebarCollapsed = false)}>
        <span class="collapsed-icon">◧</span>
        <span class="collapsed-tooltip">Show sidebar <kbd>⌘B</kbd></span>
      </div>
    {:else}
      <div class="sidebar-panel" style="width: {appState.sidebarWidth}px">
        <Sidebar onNewTerminal={handleNewTerminal} onOpenSettings={() => (showSettings = true)} onToggleSidebar={() => (appState.sidebarCollapsed = true)} onToggleRightPanel={() => (appState.rightPanelCollapsed = !appState.rightPanelCollapsed)} onOpenProfiles={() => (showProfiles = true)} />
      </div>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="resize-handle vertical-handle" onmousedown={startResizeSidebar}></div>
    {/if}
    <div class="center-panel">
      {@render terminalContent()}
    </div>
    {#if appState.rightPanelCollapsed}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="collapsed-right" onclick={() => (appState.rightPanelCollapsed = false)}>
        <span class="collapsed-icon">◨</span>
        <span class="collapsed-tooltip">Show panel <kbd>⌘J</kbd></span>
      </div>
    {:else}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="resize-handle vertical-handle" onmousedown={startResizeRight}></div>
      <div class="right-panel" style="width: {rightPanelWidth}px">
        <RightPanel onClose={() => (appState.rightPanelCollapsed = true)} />
      </div>
    {/if}
  </div>
{:else}
  <div
    class="app-layout horizontal"
    class:resizing={isResizingSidebar || isResizingRight}
  >
    <div class="horizontal-top">
      {#if appState.sidebarCollapsed}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="collapsed-sidebar" onclick={() => (appState.sidebarCollapsed = false)}>
          <span class="collapsed-icon">◧</span>
          <span class="collapsed-tooltip">Show sidebar <kbd>⌘B</kbd></span>
        </div>
      {:else}
        <div class="sidebar-panel" style="width: {appState.sidebarWidth}px">
          <Sidebar onNewTerminal={handleNewTerminal} onOpenSettings={() => (showSettings = true)} onToggleSidebar={() => (appState.sidebarCollapsed = true)} onToggleRightPanel={() => (appState.rightPanelCollapsed = !appState.rightPanelCollapsed)} onOpenProfiles={() => (showProfiles = true)} />
        </div>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="resize-handle vertical-handle" onmousedown={startResizeSidebar}></div>
      {/if}
      <div class="center-panel">
        {@render terminalContent()}
      </div>
    </div>
    {#if appState.rightPanelCollapsed}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="collapsed-bottom" onclick={() => (appState.rightPanelCollapsed = false)}>
        <span class="collapsed-icon">◫</span>
        <span class="collapsed-tooltip">Show panel <kbd>⌘J</kbd></span>
      </div>
    {:else}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="resize-handle horizontal-handle" onmousedown={startResizeRight}></div>
      <div class="bottom-panel" style="height: {rightPanelWidth}px">
        <RightPanel onClose={() => (appState.rightPanelCollapsed = true)} />
      </div>
    {/if}
  </div>
{/if}

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(html, body) {
    height: 100%;
    overflow: hidden;
    background: #0d1117;
    color: #e6edf3;
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Segoe UI', Roboto, sans-serif;
  }

  .titlebar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 38px;
    z-index: 100;
    -webkit-app-region: drag;
  }

  .app-layout {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .app-layout.vertical {
    flex-direction: row;
  }

  .app-layout.horizontal {
    flex-direction: column;
  }

  .app-layout.resizing {
    user-select: none;
  }

  .app-layout.vertical.resizing {
    cursor: col-resize;
  }

  .app-layout.horizontal.resizing {
    cursor: row-resize;
  }

  .collapsed-sidebar,
  .collapsed-right,
  .collapsed-bottom {
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 48px;
    position: relative;
    background: #1c1c1e;
    cursor: pointer;
    flex-shrink: 0;
    transition: background 0.15s;
  }

  .collapsed-sidebar:hover,
  .collapsed-right:hover,
  .collapsed-bottom:hover {
    background: #2a2a2c;
  }

  .collapsed-sidebar {
    width: 40px;
    height: 100%;
    border-right: 1px solid #2a2a2c;
  }

  .collapsed-right {
    width: 40px;
    height: 100%;
    border-left: 1px solid #30363d;
  }

  .collapsed-bottom {
    width: 100%;
    height: 40px;
    border-top: 1px solid #30363d;
    padding-top: 0;
    align-items: center;
  }

  .collapsed-icon {
    font-size: 16px;
    color: #636366;
    transition: color 0.15s;
  }

  .collapsed-sidebar:hover .collapsed-icon,
  .collapsed-right:hover .collapsed-icon,
  .collapsed-bottom:hover .collapsed-icon {
    color: #e6edf3;
  }

  .collapsed-tooltip {
    display: none;
    position: absolute;
    white-space: nowrap;
    background: #2a2a2c;
    border: 1px solid #3a3a3c;
    color: #e6edf3;
    font-size: 11px;
    padding: 4px 10px;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    z-index: 50;
    pointer-events: none;
  }

  .collapsed-tooltip kbd {
    font-family: 'SF Mono', Menlo, monospace;
    font-size: 10px;
    color: #58a6ff;
    background: #1c1c1e;
    padding: 1px 5px;
    border-radius: 3px;
    margin-left: 4px;
  }

  .collapsed-sidebar:hover .collapsed-tooltip {
    display: block;
    left: 48px;
    top: 44px;
  }

  .collapsed-right:hover .collapsed-tooltip {
    display: block;
    right: 48px;
    top: 44px;
  }

  .collapsed-bottom:hover .collapsed-tooltip {
    display: block;
    top: -32px;
    left: 50%;
    transform: translateX(-50%);
  }

  .sidebar-panel {
    flex-shrink: 0;
    height: 100%;
    overflow: visible;
    border-right: 1px solid #2a2a2c;
  }

  .center-panel {
    flex: 1;
    min-width: 200px;
    min-height: 150px;
    display: flex;
    flex-direction: column;
    position: relative;
    overflow: hidden;
  }


  /* Vertical layout */
  .right-panel {
    flex-shrink: 0;
    height: 100%;
    overflow: hidden;
    border-left: 1px solid #30363d;
  }

  /* Horizontal layout */
  .horizontal-top {
    display: flex;
    flex: 1;
    min-height: 150px;
    overflow: hidden;
  }

  .bottom-panel {
    flex-shrink: 0;
    width: 100%;
    overflow: hidden;
    border-top: 1px solid #30363d;
  }

  .resize-handle {
    background: transparent;
    transition: background 0.15s;
    flex-shrink: 0;
  }

  .resize-handle:hover {
    background: #58a6ff;
  }

  .vertical-handle {
    width: 4px;
    cursor: col-resize;
  }

  .horizontal-handle {
    height: 4px;
    cursor: row-resize;
  }

  .terminal-tab {
    display: none;
    min-height: 0;
  }

  .terminal-tab.visible {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .terminal-tabs-bar {
    display: flex;
    background: #161b22;
    border-top: 1px solid #30363d;
    padding: 0 8px;
    flex-shrink: 0;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
  }

  .terminal-tabs-bar::-webkit-scrollbar {
    display: none;
  }

  .terminal-tab-btn {
    padding: 6px 14px;
    background: none;
    border: none;
    color: #8b949e;
    cursor: pointer;
    font-size: 12px;
    border-bottom: 2px solid transparent;
    transition: all 0.15s;
  }

  .terminal-tab-btn:hover {
    color: #e6edf3;
  }

  .terminal-tab-btn.active {
    color: #e6edf3;
    border-bottom-color: #58a6ff;
  }

  .terminal-tab-btn.tab-drag-over {
    border-left: 2px solid #58a6ff;
  }

  .terminal-tab-btn.tab-dragging {
    opacity: 0.4;
  }

  .tab-shortcut {
    font-size: 10px;
    color: #484f58;
    margin-right: 4px;
  }

  .terminal-tab-btn.active .tab-shortcut {
    color: #6e7681;
  }

  .tab-rename-input {
    background: #0d1117;
    border: 1px solid #58a6ff;
    border-radius: 3px;
    color: #e6edf3;
    font-size: 12px;
    font-family: inherit;
    padding: 1px 4px;
    outline: none;
    width: 80px;
    -webkit-user-select: text;
    user-select: text;
  }

  .empty-terminal {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #0d1117;
  }

  .empty-content {
    text-align: center;
    color: #484f58;
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: 16px;
    opacity: 0.3;
  }

  .empty-content h2 {
    font-size: 24px;
    font-weight: 600;
    color: #8b949e;
    margin-bottom: 8px;
  }

  .empty-content p {
    font-size: 14px;
    color: #484f58;
    margin-bottom: 4px;
  }

  .empty-content kbd {
    display: inline-flex;
    padding: 2px 6px;
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 4px;
    font-family: 'SF Mono', Menlo, monospace;
    font-size: 12px;
    color: #8b949e;
  }
</style>
