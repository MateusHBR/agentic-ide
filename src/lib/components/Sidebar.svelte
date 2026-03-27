<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { appState } from "$lib/state.svelte";
  import type { ProjectInfo, WorktreeInfo } from "$lib/state.svelte";

  interface Props {
    onNewTerminal: (worktreePath: string) => void;
    onOpenSettings: () => void;
    onToggleSidebar: () => void;
    onToggleRightPanel: () => void;
  }
  let { onNewTerminal, onOpenSettings, onToggleSidebar, onToggleRightPanel }: Props = $props();

  let expandedProjects = $state<Set<string>>(new Set());

  function toggleProject(path: string) {
    const next = new Set(expandedProjects);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    expandedProjects = next;
  }

  function isExpanded(path: string): boolean {
    return expandedProjects.has(path);
  }

  function shortenPath(path: string): string {
    const home = "/Users/";
    if (path.startsWith(home)) {
      const rest = path.slice(home.length);
      const parts = rest.split("/");
      if (parts.length > 3) {
        return `/Users/${parts[0]}/${parts[1].slice(0, 1)}.../${parts[parts.length - 1]}`;
      }
    }
    return path;
  }

  async function addProject() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select a Git project folder",
      });
      if (selected) {
        const info = await appState.addProject(selected);
        if (info) {
          expandedProjects = new Set([...expandedProjects, info.path]);
          // Auto-open a terminal for the main worktree
          const mainWt = info.worktrees.find((w) => w.is_main) || info.worktrees[0];
          if (mainWt) {
            onNewTerminal(mainWt.path);
          }
        }
      }
    } catch (e) {
      console.error("Failed to add project:", e);
    }
  }

  function selectWorktree(project: ProjectInfo, wt: WorktreeInfo) {
    appState.selectWorktree(project.path, wt.path);
  }

  function getTerminalCount(worktreePath: string): number {
    return appState.getTerminalsForWorktree(worktreePath).length;
  }

  function isActiveWorktree(worktreePath: string): boolean {
    return appState.activeWorktree === worktreePath;
  }

  // --- Context menu ---
  let contextMenu = $state<{ x: number; y: number; projectPath: string } | null>(null);

  function showContextMenu(e: MouseEvent, projectPath: string) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, projectPath };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function removeProjectFromMenu() {
    if (contextMenu) {
      appState.removeProject(contextMenu.projectPath);
      contextMenu = null;
    }
  }

  // --- Rename ---
  let editingTerminalId = $state<string | null>(null);
  let editValue = $state("");
  let editInputEl = $state<HTMLInputElement | null>(null);

  function startRename(id: string, currentName: string) {
    editingTerminalId = id;
    editValue = currentName;
    // Focus after DOM update
    requestAnimationFrame(() => editInputEl?.focus());
  }

  function commitRename() {
    if (editingTerminalId && editValue.trim()) {
      appState.renameTerminal(editingTerminalId, editValue.trim());
    }
    editingTerminalId = null;
  }

  function cancelRename() {
    editingTerminalId = null;
  }

  // --- Drag reorder ---
  let dragTerminalId = $state<string | null>(null);
  let dragOverTerminalId = $state<string | null>(null);

  function onDragStart(e: DragEvent, id: string) {
    dragTerminalId = id;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", id);
    }
  }

  function onDragOver(e: DragEvent, id: string) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragOverTerminalId = id;
  }

  function onDragLeave() {
    dragOverTerminalId = null;
  }

  function onDrop(e: DragEvent, targetId: string) {
    e.preventDefault();
    dragOverTerminalId = null;
    if (!dragTerminalId || dragTerminalId === targetId) {
      dragTerminalId = null;
      return;
    }
    const fromIdx = appState.terminals.findIndex((t) => t.id === dragTerminalId);
    const toIdx = appState.terminals.findIndex((t) => t.id === targetId);
    if (fromIdx >= 0 && toIdx >= 0) {
      appState.reorderTerminals(fromIdx, toIdx);
    }
    dragTerminalId = null;
  }

  function onDragEnd() {
    dragTerminalId = null;
    dragOverTerminalId = null;
  }
</script>

<div class="sidebar">
  <div class="sidebar-header">
    <span class="header-title">Projects</span>
    <div class="header-actions">
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="panel-toggle-btn active" onclick={onToggleSidebar}>◧<span class="btn-tooltip">Hide sidebar <kbd>⌘B</kbd></span></span>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="panel-toggle-btn" class:active={!appState.rightPanelCollapsed} onclick={onToggleRightPanel}>◨<span class="btn-tooltip">Toggle panel <kbd>⌘J</kbd></span></span>
    </div>
  </div>

  <div class="project-list">
    {#each appState.projects as project}
      <div class="project-entry">
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="project-header"
          class:active={appState.activeProject === project.path}
          onclick={() => {
            appState.activeProject = project.path;
            if (project.worktrees.length > 0 && !project.worktrees.some(w => w.path === appState.activeWorktree)) {
              appState.selectWorktree(project.path, project.worktrees[0].path);
            }
            toggleProject(project.path);
          }}
          oncontextmenu={(e) => showContextMenu(e, project.path)}
        >
          <div class="project-icon">
            <span class="icon-letter">{project.name[0]?.toUpperCase()}</span>
          </div>
          <div class="project-info">
            <div class="project-name">{project.name}</div>
            {#if project.worktrees.length > 0}
              <div class="project-branch">
                <span class="branch-icon">⎇</span>
                {project.worktrees[0].branch}
              </div>
            {/if}
            <div class="project-path">{shortenPath(project.path)}</div>
          </div>
        </div>

        {#if isExpanded(project.path) || true}
          <div class="worktree-list">
            {#each project.worktrees as wt, i}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="worktree-item"
                class:active={isActiveWorktree(wt.path)}
                onclick={() => selectWorktree(project, wt)}
              >
                <div class="wt-indicator" class:main={wt.is_main}></div>
                <div class="wt-info">
                  <div class="wt-branch">
                    {#if wt.is_main}
                      <span class="branch-badge main">● main</span>
                    {:else}
                      <span class="branch-badge">○ {wt.branch}</span>
                    {/if}
                  </div>
                  <div class="wt-path">{shortenPath(wt.path)}</div>
                  <div class="wt-terminals">
                    <span class="terminal-count">
                      🖥 {getTerminalCount(wt.path)}
                    </span>
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <span
                      class="new-terminal-btn"
                      onclick={(e) => { e.stopPropagation(); onNewTerminal(wt.path); }}
                    >
                      ◎ New Terminal
                    </span>
                  </div>

                  {#each appState.getTerminalsForWorktree(wt.path) as term (term.id)}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div
                      class="terminal-entry"
                      class:active={appState.activeTerminalId === term.id}
                      class:drag-over={dragOverTerminalId === term.id}
                      class:dragging={dragTerminalId === term.id}
                      draggable="true"
                      onclick={(e) => { e.stopPropagation(); appState.setActiveTerminal(term.id); }}
                      ondblclick={(e) => { e.stopPropagation(); startRename(term.id, term.name); }}
                      ondragstart={(e) => onDragStart(e, term.id)}
                      ondragover={(e) => onDragOver(e, term.id)}
                      ondragleave={onDragLeave}
                      ondrop={(e) => onDrop(e, term.id)}
                      ondragend={onDragEnd}
                    >
                      <span class="terminal-icon">▶</span>
                      {#if editingTerminalId === term.id}
                        <input
                          class="rename-input"
                          type="text"
                          bind:this={editInputEl}
                          bind:value={editValue}
                          onblur={commitRename}
                          onclick={(e) => e.stopPropagation()}
                          onkeydown={(e) => {
                            if (e.key === "Enter") commitRename();
                            if (e.key === "Escape") cancelRename();
                          }}
                        />
                      {:else}
                        <span class="terminal-name">{term.name}</span>
                      {/if}
                    </div>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <div class="sidebar-footer">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="footer-btn" onclick={addProject}>
      <span class="footer-icon">+</span>
      <span>Add Project...</span>
    </div>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="footer-btn" onclick={onOpenSettings}>
      <span class="footer-icon">⚙</span>
      <span>Settings</span>
    </div>
  </div>
</div>

{#if contextMenu}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="context-overlay" onclick={closeContextMenu}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="context-menu" style="left: {contextMenu.x}px; top: {contextMenu.y}px">
      <div class="context-item danger" onclick={removeProjectFromMenu}>
        Remove Project
      </div>
    </div>
  </div>
{/if}

<style>
  .context-overlay {
    position: fixed;
    inset: 0;
    z-index: 250;
  }

  .context-menu {
    position: fixed;
    background: #2a2a2c;
    border: 1px solid #3a3a3c;
    border-radius: 8px;
    padding: 4px;
    min-width: 160px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    z-index: 251;
  }

  .context-item {
    padding: 6px 12px;
    font-size: 13px;
    color: #e6edf3;
    border-radius: 5px;
    cursor: pointer;
    transition: background 0.1s;
  }

  .context-item:hover {
    background: #3a3a3c;
  }

  .context-item.danger {
    color: #ff7b72;
  }

  .context-item.danger:hover {
    background: rgba(255, 123, 114, 0.15);
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1c1c1e;
    color: #e0e0e0;
    font-size: 13px;
    user-select: none;
  }

  .sidebar-header {
    padding: 42px 12px 8px 16px;
    border-bottom: 1px solid #2a2a2c;
    display: flex;
    align-items: center;
    justify-content: space-between;
    overflow: visible;
    position: relative;
    z-index: 10;
  }

  .header-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #888;
  }

  .header-actions {
    display: flex;
    gap: 2px;
  }

  .panel-toggle-btn {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 5px;
    cursor: pointer;
    font-size: 13px;
    color: #636366;
    transition: all 0.15s;
    position: relative;
  }

  .panel-toggle-btn:hover {
    color: #e6edf3;
    background: #3a3a3c;
  }

  .panel-toggle-btn.active {
    color: #58a6ff;
  }

  .btn-tooltip {
    display: none;
    position: absolute;
    top: 32px;
    right: 0;
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

  .btn-tooltip kbd {
    font-family: 'SF Mono', Menlo, monospace;
    font-size: 10px;
    color: #58a6ff;
    background: #1c1c1e;
    padding: 1px 5px;
    border-radius: 3px;
    margin-left: 4px;
  }

  .panel-toggle-btn:hover .btn-tooltip {
    display: block;
  }

  .project-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .project-entry {
    border-bottom: 1px solid #2a2a2c;
  }

  .project-header {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 12px;
    cursor: pointer;
    transition: background 0.15s;
  }

  .project-header:hover {
    background: #2a2a2c;
  }

  .project-header.active {
    background: #2a2a2c;
  }

  .project-icon {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    background: #3a3a3c;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: background 0.15s;
  }


  .icon-letter {
    font-size: 16px;
    font-weight: 700;
    color: #fff;
  }

  .project-info {
    flex: 1;
    min-width: 0;
  }

  .project-name {
    font-weight: 600;
    color: #fff;
    margin-bottom: 2px;
  }

  .project-branch {
    font-size: 11px;
    color: #8e8e93;
    margin-bottom: 2px;
  }

  .branch-icon {
    font-size: 12px;
  }

  .project-path {
    font-size: 10px;
    color: #636366;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .worktree-list {
    padding: 0 8px 4px 8px;
  }

  .worktree-item {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 6px 8px;
    margin: 2px 0;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.15s;
  }

  .worktree-item:hover {
    background: #2a2a2c;
  }

  .worktree-item.active {
    background: #122b17;
    border-left: 3px solid #30d158;
    padding-left: 5px;
  }

  .wt-indicator {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #636366;
    margin-top: 6px;
    flex-shrink: 0;
  }

  .wt-indicator.main {
    background: #30d158;
  }

  .worktree-item.active .wt-indicator {
    background: #30d158;
    box-shadow: 0 0 6px rgba(48, 209, 88, 0.5);
  }

  .wt-info {
    flex: 1;
    min-width: 0;
  }

  .wt-branch {
    margin-bottom: 2px;
  }

  .branch-badge {
    font-size: 12px;
    color: #aaa;
  }

  .branch-badge.main {
    color: #30d158;
  }

  .worktree-item.active .branch-badge {
    color: #30d158;
  }

  .worktree-item.active .wt-path {
    color: #2a9d48;
  }

  .wt-path {
    font-size: 10px;
    color: #636366;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 3px;
  }

  .wt-terminals {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 11px;
    color: #8e8e93;
  }

  .new-terminal-btn {
    cursor: pointer;
    transition: color 0.15s;
  }

  .new-terminal-btn:hover {
    color: #fff;
  }

  .terminal-entry {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    margin: 2px 0;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    color: #8e8e93;
    transition: background 0.15s;
  }

  .terminal-entry:hover {
    background: #2a2a2c;
  }

  .terminal-entry.active {
    background: #323236;
    color: #fff;
  }

  .terminal-entry.drag-over {
    border-top: 2px solid #58a6ff;
  }

  .terminal-entry.dragging {
    opacity: 0.4;
  }

  .terminal-icon {
    font-size: 10px;
    color: #30d158;
  }

  .rename-input {
    background: #1a1a1c;
    border: 1px solid #58a6ff;
    border-radius: 3px;
    color: #fff;
    font-size: 12px;
    font-family: inherit;
    padding: 1px 4px;
    outline: none;
    width: 100%;
    -webkit-user-select: text;
    user-select: text;
  }

  .sidebar-footer {
    border-top: 1px solid #2a2a2c;
  }

  .footer-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    cursor: pointer;
    color: #8e8e93;
    font-size: 13px;
    transition: background 0.15s;
  }

  .footer-btn:hover {
    background: #2a2a2c;
    color: #fff;
  }

  .footer-icon {
    font-size: 16px;
    line-height: 1;
    width: 18px;
    text-align: center;
  }
</style>
