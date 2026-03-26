<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "$lib/state.svelte";
  import type { RightTab } from "$lib/state.svelte";

  interface Props {
    onClose?: () => void;
  }
  let { onClose }: Props = $props();

  const tabs: { id: RightTab; label: string }[] = [
    { id: "info", label: "Info" },
    { id: "timeline", label: "Timeline" },
    { id: "agent", label: "Agent" },
    { id: "changes", label: "Changes" },
  ];

  function selectTab(tab: RightTab) {
    appState.rightTab = tab;
    appState.refreshRightPanel();
  }

  function formatStatus(status: string): string {
    const s = status.trim();
    if (s === "M " || s === " M" || s === "MM") return "Modified";
    if (s === "A " || s === " A") return "Added";
    if (s === "D " || s === " D") return "Deleted";
    if (s === "R ") return "Renamed";
    if (s === "??") return "Untracked";
    return s;
  }

  function statusColor(status: string): string {
    const s = status.trim();
    if (s.includes("M")) return "#d29922";
    if (s.includes("A") || s === "??") return "#3fb950";
    if (s.includes("D")) return "#ff7b72";
    if (s.includes("R")) return "#58a6ff";
    return "#8b949e";
  }

  // --- Collapse state ---
  let unstagedCollapsed = $state(false);
  let stagedCollapsed = $state(false);
  let collapsedFiles = $state<Set<string>>(new Set());

  function toggleCollapse(file: string) {
    const next = new Set(collapsedFiles);
    if (next.has(file)) {
      next.delete(file);
    } else {
      next.add(file);
    }
    collapsedFiles = next;
  }

  function isCollapsed(file: string): boolean {
    return collapsedFiles.has(file);
  }

  // --- Stage / unstage ---
  function isFileStaged(file: string): boolean {
    const entry = appState.gitStatus.find((f) => f.file === file);
    if (!entry) return false;
    const idx = entry.status[0];
    return idx !== " " && idx !== "?";
  }

  async function stageFile(file: string) {
    if (!appState.activeWorktree) return;
    try {
      await invoke("stage_file", { worktreePath: appState.activeWorktree, file });
      await appState.refreshRightPanel();
    } catch (e) {
      console.error("Failed to stage file:", e);
    }
  }

  async function unstageFile(file: string) {
    if (!appState.activeWorktree) return;
    try {
      await invoke("unstage_file", { worktreePath: appState.activeWorktree, file });
      await appState.refreshRightPanel();
    } catch (e) {
      console.error("Failed to unstage file:", e);
    }
  }

  async function toggleStage(file: string) {
    if (isFileStaged(file)) {
      await unstageFile(file);
    } else {
      await stageFile(file);
    }
  }

  async function stageAll() {
    if (!appState.activeWorktree) return;
    try {
      await invoke("stage_file", { worktreePath: appState.activeWorktree, file: "." });
      await appState.refreshRightPanel();
    } catch (e) {
      console.error("Failed to stage all:", e);
    }
  }

  async function unstageAll() {
    if (!appState.activeWorktree) return;
    try {
      await invoke("unstage_file", { worktreePath: appState.activeWorktree, file: "." });
      await appState.refreshRightPanel();
    } catch (e) {
      console.error("Failed to unstage all:", e);
    }
  }

  const IMAGE_EXTENSIONS = new Set([
    "png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico", "tiff", "tif", "avif",
  ]);

  function isImageFile(file: string): boolean {
    const ext = file.split(".").pop()?.toLowerCase() ?? "";
    return IMAGE_EXTENSIONS.has(ext);
  }

  function getMimeType(file: string): string {
    const ext = file.split(".").pop()?.toLowerCase() ?? "";
    const mimes: Record<string, string> = {
      png: "image/png", jpg: "image/jpeg", jpeg: "image/jpeg",
      gif: "image/gif", bmp: "image/bmp", webp: "image/webp",
      svg: "image/svg+xml", ico: "image/x-icon", tiff: "image/tiff",
      tif: "image/tiff", avif: "image/avif",
    };
    return mimes[ext] ?? "image/png";
  }

  let imageCache = $state<Map<string, { current?: string; previous?: string }>>(new Map());

  async function loadImagePreview(file: string) {
    if (!appState.activeWorktree || imageCache.has(file)) return;
    const mime = getMimeType(file);
    const entry: { current?: string; previous?: string } = {};
    try {
      const b64: string = await invoke("read_file_base64", {
        worktreePath: appState.activeWorktree, file,
      });
      entry.current = `data:${mime};base64,${b64}`;
    } catch (_) {}
    try {
      const b64: string = await invoke("read_git_file_base64", {
        worktreePath: appState.activeWorktree, file,
      });
      entry.previous = `data:${mime};base64,${b64}`;
    } catch (_) {}
    const updated = new Map(imageCache);
    updated.set(file, entry);
    imageCache = updated;
  }

  interface DiffLine {
    type: string;
    content: string;
    oldNum: number | null;
    newNum: number | null;
  }

  interface DiffHunk {
    header: string;
    lines: DiffLine[];
    oldStart: number;
    newStart: number;
  }

  interface DiffFile {
    file: string;
    hunks: DiffHunk[];
  }

  function parseDiff(diff: string): DiffFile[] {
    if (!diff) return [];
    const files: DiffFile[] = [];
    const fileSections = diff.split(/^diff --git /m).filter(Boolean);

    for (const section of fileSections) {
      const lines = section.split("\n");
      const fileMatch = lines[0]?.match(/a\/(.*?) b\/(.*)/);
      const fileName = fileMatch ? fileMatch[2] : "unknown";

      const hunks: DiffHunk[] = [];
      let currentHunk: DiffHunk | null = null;
      let oldLine = 0;
      let newLine = 0;

      for (const line of lines.slice(1)) {
        if (line.startsWith("@@")) {
          if (currentHunk) hunks.push(currentHunk);
          const match = line.match(/@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
          oldLine = match ? parseInt(match[1]) : 0;
          newLine = match ? parseInt(match[2]) : 0;
          currentHunk = { header: line, lines: [], oldStart: oldLine, newStart: newLine };
        } else if (currentHunk) {
          if (line.startsWith("+")) {
            currentHunk.lines.push({ type: "add", content: line, oldNum: null, newNum: newLine });
            newLine++;
          } else if (line.startsWith("-")) {
            currentHunk.lines.push({ type: "remove", content: line, oldNum: oldLine, newNum: null });
            oldLine++;
          } else {
            currentHunk.lines.push({ type: "context", content: line, oldNum: oldLine, newNum: newLine });
            oldLine++;
            newLine++;
          }
        }
      }
      if (currentHunk) hunks.push(currentHunk);
      if (hunks.length > 0) {
        files.push({ file: fileName, hunks });
      }
    }
    return files;
  }

  // --- Expand context ---
  let expandedContexts = $state<Map<string, number>>(new Map());
  let expandedOverrides = $state<Map<string, DiffFile>>(new Map());

  function getContextLines(key: string): number {
    return expandedContexts.get(key) ?? 3;
  }

  async function expandContext(file: string, staged: boolean, direction: "up" | "down" | "all") {
    if (!appState.activeWorktree) return;
    const key = (staged ? "staged:" : "unstaged:") + file;
    const current = getContextLines(key);
    const next = direction === "all" ? 99999 : current + 20;

    try {
      const newDiff: string = await invoke("get_file_diff", {
        worktreePath: appState.activeWorktree,
        file,
        contextLines: next,
        staged,
      });
      const parsed = parseDiff(newDiff);
      if (parsed.length > 0) {
        const updatedCtx = new Map(expandedContexts);
        updatedCtx.set(key, next);
        expandedContexts = updatedCtx;

        const updatedOverrides = new Map(expandedOverrides);
        updatedOverrides.set(key, parsed[0]);
        expandedOverrides = updatedOverrides;
      }
    } catch (e) {
      console.error("Failed to expand context:", e);
    }
  }

  function canExpandUp(hunk: DiffHunk): boolean {
    return hunk.oldStart > 1;
  }

  function isFullyExpanded(file: string, staged: boolean): boolean {
    const key = (staged ? "staged:" : "unstaged:") + file;
    return (expandedContexts.get(key) ?? 3) >= 99999;
  }

  function resetExpand(file: string, staged: boolean) {
    const key = (staged ? "staged:" : "unstaged:") + file;
    const updatedCtx = new Map(expandedContexts);
    updatedCtx.delete(key);
    expandedContexts = updatedCtx;

    const updatedOverrides = new Map(expandedOverrides);
    updatedOverrides.delete(key);
    expandedOverrides = updatedOverrides;
  }

  let baseUnstagedFiles = $derived(parseDiff(appState.diff));
  let baseStagedFiles = $derived(parseDiff(appState.stagedDiff));

  let unstagedFiles = $derived(
    baseUnstagedFiles.map((f) => {
      const override = expandedOverrides.get("unstaged:" + f.file);
      return override ?? f;
    })
  );

  let stagedFiles = $derived(
    baseStagedFiles.map((f) => {
      const override = expandedOverrides.get("staged:" + f.file);
      return override ?? f;
    })
  );

  // Unstaged-only status entries (untracked files etc. not in diff)
  let unstagedStatusOnly = $derived(
    appState.gitStatus.filter(
      (f) => {
        const idx = f.status[0];
        const wt = f.status[1];
        const isUnstaged = wt !== " " || f.status === "??";
        return isUnstaged && !unstagedFiles.some((d) => d.file === f.file);
      }
    )
  );

  let stagedStatusOnly = $derived(
    appState.gitStatus.filter(
      (f) => {
        const idx = f.status[0];
        const isStaged = idx !== " " && idx !== "?";
        return isStaged && !stagedFiles.some((d) => d.file === f.file);
      }
    )
  );

  let hasUnstaged = $derived(unstagedFiles.length > 0 || unstagedStatusOnly.length > 0);
  let hasStaged = $derived(stagedFiles.length > 0 || stagedStatusOnly.length > 0);

  // Auto-load image previews for image files in status
  $effect(() => {
    const allFiles = appState.gitStatus;
    for (const f of allFiles) {
      if (isImageFile(f.file) && !imageCache.has(f.file)) {
        loadImagePreview(f.file);
      }
    }
  });
</script>

<div class="right-panel">
  <div class="tab-bar">
    {#each tabs as tab}
      <button
        class="tab"
        class:active={appState.rightTab === tab.id}
        onclick={() => selectTab(tab.id)}
      >
        {tab.label}
      </button>
    {/each}
    <span class="tab-spacer"></span>
    {#if onClose}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="panel-close-btn" onclick={onClose}>◨<span class="close-tooltip">Hide panel <kbd>⌘J</kbd></span></span>
    {/if}
  </div>

  <div class="tab-content">
    {#if appState.rightTab === "info"}
      <div class="info-panel">
        {#if appState.activeWorktree}
          <div class="info-section">
            <h3>Worktree</h3>
            <div class="info-row">
              <span class="info-label">Path</span>
              <span class="info-value">{appState.activeWorktree}</span>
            </div>
            {#each appState.projects as project}
              {#each project.worktrees.filter(w => w.path === appState.activeWorktree) as wt}
                <div class="info-row">
                  <span class="info-label">Branch</span>
                  <span class="info-value branch">{wt.branch}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">HEAD</span>
                  <span class="info-value mono">{wt.head.slice(0, 12)}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">Type</span>
                  <span class="info-value">{wt.is_main ? "Main worktree" : "Linked worktree"}</span>
                </div>
              {/each}
            {/each}
          </div>

          <div class="info-section">
            <h3>Status</h3>
            {#if appState.gitStatus.length === 0}
              <p class="empty-message">Working tree clean</p>
            {:else}
              <div class="file-status-list">
                {#each appState.gitStatus as file}
                  <div class="file-status-item">
                    <span class="status-badge" style="color: {statusColor(file.status)}">
                      {formatStatus(file.status)}
                    </span>
                    <span class="file-name">{file.file}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {:else}
          <p class="empty-message">Select a worktree to view info</p>
        {/if}
      </div>

    {:else if appState.rightTab === "timeline"}
      <div class="timeline-panel">
        {#if appState.gitLog.length === 0}
          <p class="empty-message">No commits found</p>
        {:else}
          {#each appState.gitLog as entry}
            <div class="timeline-entry">
              <div class="timeline-dot"></div>
              <div class="timeline-content">
                <div class="commit-message">{entry.message}</div>
                <div class="commit-meta">
                  <span class="commit-hash">{entry.short_hash}</span>
                  <span class="commit-author">{entry.author}</span>
                  <span class="commit-time">{entry.relative_time}</span>
                </div>
              </div>
            </div>
          {/each}
        {/if}
      </div>

    {:else if appState.rightTab === "agent"}
      <div class="agent-panel">
        <div class="agent-header">
          <h3>Claude Code Session</h3>
        </div>
        <div class="agent-info">
          {#if appState.activeTerminalId}
            <div class="agent-status active">
              <span class="status-dot"></span>
              Active Session
            </div>
            <p class="agent-description">
              Claude Code is running in the terminal. Changes will appear in the Changes and Timeline tabs.
            </p>
            <button class="refresh-btn" onclick={() => appState.refreshRightPanel()}>
              Refresh Changes
            </button>
          {:else}
            <div class="agent-status inactive">
              <span class="status-dot"></span>
              No Active Session
            </div>
            <p class="agent-description">
              Create a new terminal to start a Claude Code session.
            </p>
          {/if}
        </div>
      </div>

    {:else if appState.rightTab === "changes"}
      <div class="changes-panel">
        {#if !hasUnstaged && !hasStaged}
          <p class="empty-message">No changes detected</p>
        {:else}
          <!-- Unstaged Changes -->
          {#if hasUnstaged}
            <div class="changes-section">
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="section-header" onclick={() => (unstagedCollapsed = !unstagedCollapsed)}>
                <span class="section-collapse">{unstagedCollapsed ? "▶" : "▼"}</span>
                <span class="section-title">Unstaged Changes</span>
                <span class="section-count">{unstagedFiles.length + unstagedStatusOnly.length}</span>
                <span class="section-spacer"></span>
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span class="stage-all-btn" onclick={(e) => { e.stopPropagation(); stageAll(); }}>Stage All</span>
              </div>

              {#if !unstagedCollapsed}
              {#each unstagedFiles as file}
                <div class="diff-file">
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="diff-file-header" onclick={() => toggleCollapse("unstaged:" + file.file)}>
                    <span class="collapse-icon">{isCollapsed("unstaged:" + file.file) ? "▶" : "▼"}</span>
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <span
                      class="stage-checkbox"
                      onclick={(e) => { e.stopPropagation(); stageFile(file.file); }}
                    >
                    </span>
                    <span class="diff-file-name">{file.file}</span>
                    <span class="diff-file-lines">
                      <span class="additions">+{file.hunks.reduce((sum, h) => sum + h.lines.filter(l => l.type === "add").length, 0)}</span>
                      <span class="deletions">-{file.hunks.reduce((sum, h) => sum + h.lines.filter(l => l.type === "remove").length, 0)}</span>
                    </span>
                  </div>
                  {#if !isCollapsed("unstaged:" + file.file)}
                    {#each file.hunks as hunk, hunkIdx}
                      <div class="diff-hunk">
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div class="hunk-header">
                          {#if isFullyExpanded(file.file, false)}
                            <span class="expand-btn" onclick={() => resetExpand(file.file, false)} title="Collapse to default context">⇕</span>
                          {:else}
                            {#if canExpandUp(hunk)}
                              <span class="expand-btn" onclick={() => expandContext(file.file, false, "up")} title="Load more lines above">↑</span>
                            {/if}
                            <span class="expand-btn" onclick={() => expandContext(file.file, false, "all")} title="Show all lines">⇕</span>
                          {/if}
                          <span class="hunk-range">{hunk.header}</span>
                        </div>
                        {#each hunk.lines as line}
                          <div
                            class="diff-line"
                            class:add={line.type === "add"}
                            class:remove={line.type === "remove"}
                          >
                            <span class="line-num old">{line.oldNum ?? ""}</span>
                            <span class="line-num new">{line.newNum ?? ""}</span>
                            <span class="diff-marker">
                              {line.type === "add" ? "+" : line.type === "remove" ? "-" : " "}
                            </span>
                            <span class="diff-content">{line.content.slice(1)}</span>
                          </div>
                        {/each}
                        {#if !isFullyExpanded(file.file, false) && hunkIdx === file.hunks.length - 1}
                          <!-- svelte-ignore a11y_click_events_have_key_events -->
                          <!-- svelte-ignore a11y_no_static_element_interactions -->
                          <div class="expand-row" onclick={() => expandContext(file.file, false, "down")} title="Load more lines below">
                            <span class="expand-btn">↓</span>
                            <span class="expand-label">Load more lines</span>
                          </div>
                        {/if}
                      </div>
                    {/each}
                  {/if}
                </div>
              {/each}

              {#each unstagedStatusOnly as file}
                <div class="diff-file">
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="diff-file-header" onclick={() => {
                    if (isImageFile(file.file)) {
                      toggleCollapse("unstaged:" + file.file);
                      loadImagePreview(file.file);
                    } else {
                      stageFile(file.file);
                    }
                  }}>
                    <span class="collapse-icon">{isImageFile(file.file) ? (isCollapsed("unstaged:" + file.file) ? "▶" : "▼") : ""}</span>
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <span class="stage-checkbox" onclick={(e) => { e.stopPropagation(); stageFile(file.file); }}></span>
                    <span class="diff-file-name">{file.file}</span>
                    <span class="status-label" style="color: {statusColor(file.status)}">{formatStatus(file.status)}</span>
                  </div>
                  {#if isImageFile(file.file) && !isCollapsed("unstaged:" + file.file) && imageCache.has(file.file)}
                    <div class="image-preview">
                      {#if imageCache.get(file.file)?.previous}
                        <div class="image-side">
                          <span class="image-label removed">Previous</span>
                          <img src={imageCache.get(file.file)?.previous} alt="Previous version" />
                        </div>
                      {/if}
                      {#if imageCache.get(file.file)?.current}
                        <div class="image-side">
                          <span class="image-label added">Current</span>
                          <img src={imageCache.get(file.file)?.current} alt="Current version" />
                        </div>
                      {/if}
                    </div>
                  {/if}
                </div>
              {/each}
              {/if}
            </div>
          {/if}

          <!-- Staged Changes -->
          {#if hasStaged}
            <div class="changes-section">
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="section-header" onclick={() => (stagedCollapsed = !stagedCollapsed)}>
                <span class="section-collapse">{stagedCollapsed ? "▶" : "▼"}</span>
                <span class="section-title">Staged Changes</span>
                <span class="section-count">{stagedFiles.length + stagedStatusOnly.length}</span>
                <span class="section-spacer"></span>
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span class="unstage-all-btn" onclick={(e) => { e.stopPropagation(); unstageAll(); }}>Unstage All</span>
              </div>

              {#if !stagedCollapsed}
              {#each stagedFiles as file}
                <div class="diff-file staged">
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="diff-file-header" onclick={() => toggleCollapse("staged:" + file.file)}>
                    <span class="collapse-icon">{isCollapsed("staged:" + file.file) ? "▶" : "▼"}</span>
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <span
                      class="stage-checkbox staged"
                      onclick={(e) => { e.stopPropagation(); unstageFile(file.file); }}
                    >
                      <span class="check-mark">✓</span>
                    </span>
                    <span class="diff-file-name">{file.file}</span>
                    <span class="diff-file-lines">
                      <span class="additions">+{file.hunks.reduce((sum, h) => sum + h.lines.filter(l => l.type === "add").length, 0)}</span>
                      <span class="deletions">-{file.hunks.reduce((sum, h) => sum + h.lines.filter(l => l.type === "remove").length, 0)}</span>
                    </span>
                  </div>
                  {#if !isCollapsed("staged:" + file.file)}
                    {#each file.hunks as hunk, hunkIdx}
                      <div class="diff-hunk">
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div class="hunk-header">
                          {#if isFullyExpanded(file.file, true)}
                            <span class="expand-btn" onclick={() => resetExpand(file.file, true)} title="Collapse to default context">⇕</span>
                          {:else}
                            {#if canExpandUp(hunk)}
                              <span class="expand-btn" onclick={() => expandContext(file.file, true, "up")} title="Load more lines above">↑</span>
                            {/if}
                            <span class="expand-btn" onclick={() => expandContext(file.file, true, "all")} title="Show all lines">⇕</span>
                          {/if}
                          <span class="hunk-range">{hunk.header}</span>
                        </div>
                        {#each hunk.lines as line}
                          <div
                            class="diff-line"
                            class:add={line.type === "add"}
                            class:remove={line.type === "remove"}
                          >
                            <span class="line-num old">{line.oldNum ?? ""}</span>
                            <span class="line-num new">{line.newNum ?? ""}</span>
                            <span class="diff-marker">
                              {line.type === "add" ? "+" : line.type === "remove" ? "-" : " "}
                            </span>
                            <span class="diff-content">{line.content.slice(1)}</span>
                          </div>
                        {/each}
                        {#if !isFullyExpanded(file.file, true) && hunkIdx === file.hunks.length - 1}
                          <!-- svelte-ignore a11y_click_events_have_key_events -->
                          <!-- svelte-ignore a11y_no_static_element_interactions -->
                          <div class="expand-row" onclick={() => expandContext(file.file, true, "down")} title="Load more lines below">
                            <span class="expand-btn">↓</span>
                            <span class="expand-label">Load more lines</span>
                          </div>
                        {/if}
                      </div>
                    {/each}
                  {/if}
                </div>
              {/each}

              {#each stagedStatusOnly as file}
                <div class="diff-file staged">
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="diff-file-header" onclick={() => {
                    if (isImageFile(file.file)) {
                      toggleCollapse("staged:" + file.file);
                      loadImagePreview(file.file);
                    } else {
                      unstageFile(file.file);
                    }
                  }}>
                    <span class="collapse-icon">{isImageFile(file.file) ? (isCollapsed("staged:" + file.file) ? "▶" : "▼") : ""}</span>
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <span class="stage-checkbox staged" onclick={(e) => { e.stopPropagation(); unstageFile(file.file); }}>
                      <span class="check-mark">✓</span>
                    </span>
                    <span class="diff-file-name">{file.file}</span>
                    <span class="status-label" style="color: {statusColor(file.status)}">{formatStatus(file.status)}</span>
                  </div>
                  {#if isImageFile(file.file) && !isCollapsed("staged:" + file.file) && imageCache.has(file.file)}
                    <div class="image-preview">
                      {#if imageCache.get(file.file)?.previous}
                        <div class="image-side">
                          <span class="image-label removed">Previous</span>
                          <img src={imageCache.get(file.file)?.previous} alt="Previous version" />
                        </div>
                      {/if}
                      {#if imageCache.get(file.file)?.current}
                        <div class="image-side">
                          <span class="image-label added">Current</span>
                          <img src={imageCache.get(file.file)?.current} alt="Current version" />
                        </div>
                      {/if}
                    </div>
                  {/if}
                </div>
              {/each}
              {/if}
            </div>
          {/if}
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .right-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #161b22;
    color: #e6edf3;
    font-size: 13px;
    overflow: hidden;
  }

  .tab-bar {
    display: flex;
    align-items: center;
    border-bottom: 1px solid #30363d;
    padding: 0 8px;
    gap: 0;
    flex-shrink: 0;
    padding-top: 32px;
  }

  .tab-spacer {
    flex: 1;
  }

  .panel-close-btn {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 5px;
    cursor: pointer;
    font-size: 13px;
    color: #58a6ff;
    transition: all 0.15s;
    margin-right: 4px;
    position: relative;
  }

  .panel-close-btn:hover {
    color: #e6edf3;
    background: #30363d;
  }

  .close-tooltip {
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

  .close-tooltip kbd {
    font-family: 'SF Mono', Menlo, monospace;
    font-size: 10px;
    color: #58a6ff;
    background: #1c1c1e;
    padding: 1px 5px;
    border-radius: 3px;
    margin-left: 4px;
  }

  .panel-close-btn:hover .close-tooltip {
    display: block;
  }

  .tab {
    padding: 8px 16px;
    background: none;
    border: none;
    color: #8b949e;
    cursor: pointer;
    font-size: 13px;
    border-bottom: 2px solid transparent;
    transition: all 0.15s;
  }

  .tab:hover {
    color: #e6edf3;
  }

  .tab.active {
    color: #58a6ff;
    border-bottom-color: #58a6ff;
  }

  .tab-content {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
  }

  /* Info Panel */
  .info-section {
    margin-bottom: 20px;
  }

  .info-section h3 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #8b949e;
    margin: 0 0 8px 0;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    padding: 4px 0;
    border-bottom: 1px solid #21262d;
  }

  .info-label {
    color: #8b949e;
  }

  .info-value {
    color: #e6edf3;
    text-align: right;
    max-width: 60%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .info-value.branch {
    color: #58a6ff;
  }

  .info-value.mono {
    font-family: 'SF Mono', Menlo, monospace;
    font-size: 12px;
  }

  .file-status-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .file-status-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 0;
  }

  .status-badge {
    font-size: 11px;
    font-weight: 600;
    min-width: 70px;
  }

  .file-name {
    font-family: 'SF Mono', Menlo, monospace;
    font-size: 12px;
    color: #e6edf3;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Timeline Panel */
  .timeline-panel {
    padding: 4px 0;
  }

  .timeline-entry {
    display: flex;
    gap: 12px;
    padding: 8px 0;
    position: relative;
  }

  .timeline-entry::before {
    content: "";
    position: absolute;
    left: 5px;
    top: 22px;
    bottom: -8px;
    width: 1px;
    background: #30363d;
  }

  .timeline-entry:last-child::before {
    display: none;
  }

  .timeline-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #30363d;
    border: 2px solid #58a6ff;
    flex-shrink: 0;
    margin-top: 4px;
  }

  .timeline-content {
    flex: 1;
    min-width: 0;
  }

  .commit-message {
    color: #e6edf3;
    margin-bottom: 4px;
    line-height: 1.4;
  }

  .commit-meta {
    display: flex;
    gap: 8px;
    font-size: 11px;
    color: #8b949e;
    flex-wrap: wrap;
  }

  .commit-hash {
    font-family: 'SF Mono', Menlo, monospace;
    color: #58a6ff;
  }

  /* Agent Panel */
  .agent-panel {
    padding: 4px 0;
  }

  .agent-header h3 {
    font-size: 14px;
    font-weight: 600;
    color: #e6edf3;
    margin: 0 0 12px 0;
  }

  .agent-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 12px;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .agent-status.active .status-dot {
    background: #3fb950;
    box-shadow: 0 0 6px rgba(63, 185, 80, 0.4);
  }

  .agent-status.inactive .status-dot {
    background: #484f58;
  }

  .agent-description {
    color: #8b949e;
    line-height: 1.5;
    margin: 0 0 16px 0;
  }

  .refresh-btn {
    padding: 6px 12px;
    background: #21262d;
    border: 1px solid #30363d;
    color: #e6edf3;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    transition: background 0.15s;
  }

  .refresh-btn:hover {
    background: #30363d;
  }

  /* Changes Panel */
  .changes-panel {
    font-family: 'SF Mono', 'Fira Code', Menlo, monospace;
    font-size: 12px;
  }

  .changes-section {
    margin-bottom: 16px;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 4px;
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', sans-serif;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.15s;
  }

  .section-header:hover {
    background: #1c2128;
  }

  .section-collapse {
    font-size: 10px;
    color: #8b949e;
    width: 12px;
    flex-shrink: 0;
  }

  .section-spacer {
    flex: 1;
  }

  .stage-all-btn {
    font-size: 11px;
    font-weight: 600;
    color: #58a6ff;
    cursor: pointer;
    padding: 2px 8px;
    border-radius: 4px;
    transition: background 0.15s;
  }

  .stage-all-btn:hover {
    background: rgba(88, 166, 255, 0.15);
  }

  .unstage-all-btn {
    font-size: 11px;
    font-weight: 600;
    color: #ff7b72;
    cursor: pointer;
    padding: 2px 8px;
    border-radius: 4px;
    transition: background 0.15s;
  }

  .unstage-all-btn:hover {
    background: rgba(255, 123, 114, 0.15);
  }

  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #8b949e;
  }

  .section-count {
    font-size: 11px;
    font-weight: 600;
    color: #8b949e;
    background: #21262d;
    padding: 1px 6px;
    border-radius: 10px;
  }

  .diff-file.staged {
    border-color: #1f3a27;
  }

  .diff-file {
    margin-bottom: 16px;
    border: 1px solid #30363d;
    border-radius: 6px;
    overflow: hidden;
  }

  .diff-file-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: #161b22;
    border-bottom: 1px solid #30363d;
    font-weight: 600;
    color: #e6edf3;
    cursor: pointer;
    transition: background 0.15s;
  }

  .diff-file-header:hover {
    background: #1c2128;
  }

  .collapse-icon {
    font-size: 10px;
    color: #8b949e;
    width: 12px;
    flex-shrink: 0;
  }

  .stage-checkbox {
    width: 16px;
    height: 16px;
    border: 1px solid #484f58;
    border-radius: 3px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s;
  }

  .stage-checkbox:hover {
    border-color: #58a6ff;
  }

  .stage-checkbox.staged {
    background: #58a6ff;
    border-color: #58a6ff;
  }

  .check-mark {
    font-size: 11px;
    color: #fff;
    line-height: 1;
  }

  .diff-file-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .diff-file-lines {
    display: flex;
    gap: 6px;
    font-size: 11px;
    font-weight: 500;
    flex-shrink: 0;
  }

  .additions {
    color: #3fb950;
  }

  .deletions {
    color: #ff7b72;
  }

  .status-label {
    font-size: 11px;
    font-weight: 500;
    flex-shrink: 0;
  }

  .diff-hunk {
    border-top: 1px solid #21262d;
  }

  .hunk-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 12px;
    background: rgba(56, 139, 253, 0.1);
    color: #8b949e;
    font-size: 11px;
  }

  .hunk-range {
    flex: 1;
  }

  .expand-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 18px;
    border-radius: 3px;
    cursor: pointer;
    color: #58a6ff;
    font-size: 12px;
    transition: background 0.15s;
    flex-shrink: 0;
  }

  .expand-btn:hover {
    background: rgba(88, 166, 255, 0.2);
  }

  .expand-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 4px 12px;
    background: rgba(56, 139, 253, 0.05);
    cursor: pointer;
    transition: background 0.15s;
  }

  .expand-row:hover {
    background: rgba(56, 139, 253, 0.15);
  }

  .expand-label {
    font-size: 11px;
    color: #58a6ff;
  }

  .diff-line {
    display: flex;
    padding: 0 12px;
    line-height: 1.6;
    white-space: pre;
    overflow-x: auto;
  }

  .diff-line.add {
    background: rgba(63, 185, 80, 0.15);
    color: #3fb950;
  }

  .diff-line.remove {
    background: rgba(248, 81, 73, 0.15);
    color: #ff7b72;
  }

  .line-num {
    width: 40px;
    flex-shrink: 0;
    text-align: right;
    padding-right: 6px;
    color: #484f58;
    font-size: 11px;
    border-right: 1px solid #21262d;
    -webkit-user-select: none;
    user-select: none;
  }

  .line-num.new {
    margin-right: 4px;
  }

  .diff-line.add .line-num.old {
    color: transparent;
  }

  .diff-line.remove .line-num.new {
    color: transparent;
  }

  .diff-marker {
    width: 16px;
    flex-shrink: 0;
    text-align: center;
    user-select: none;
  }

  .diff-content {
    flex: 1;
    min-width: 0;
  }

  .image-preview {
    display: flex;
    gap: 8px;
    padding: 12px;
    background: #0d1117;
    border-top: 1px solid #21262d;
  }

  .image-side {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .image-side img {
    max-width: 100%;
    max-height: 200px;
    object-fit: contain;
    border-radius: 4px;
    border: 1px solid #30363d;
    background: repeating-conic-gradient(#1c1c1e 0% 25%, #2a2a2c 0% 50%) 50% / 16px 16px;
  }

  .image-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 2px 8px;
    border-radius: 4px;
  }

  .image-label.removed {
    color: #ff7b72;
    background: rgba(248, 81, 73, 0.1);
  }

  .image-label.added {
    color: #3fb950;
    background: rgba(63, 185, 80, 0.1);
  }

  .empty-message {
    color: #484f58;
    text-align: center;
    padding: 40px 20px;
    font-style: italic;
  }
</style>
