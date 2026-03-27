import { invoke } from "@tauri-apps/api/core";
import { updateProfileSettings } from "$lib/profiles.svelte";
import type { ProfileSettings } from "$lib/profiles.svelte";
import { profileState } from "$lib/profiles.svelte";

export interface WorktreeInfo {
  path: string;
  branch: string;
  head: string;
  is_main: boolean;
  is_bare: boolean;
}

export interface ProjectInfo {
  name: string;
  path: string;
  worktrees: WorktreeInfo[];
}

export interface TerminalInfo {
  id: string;
  name: string;
  worktreePath: string;
}

export interface LogEntry {
  hash: string;
  short_hash: string;
  author: string;
  relative_time: string;
  message: string;
}

export interface FileStatus {
  status: string;
  file: string;
}

export type RightTab = "info" | "timeline" | "agent" | "changes";
export type LayoutMode = "vertical" | "horizontal";

class AppState {
  projects = $state<ProjectInfo[]>([]);
  activeWorktree = $state<string>("");
  activeProject = $state<string>("");
  terminals = $state<TerminalInfo[]>([]);
  activeTerminalId = $state<string>("");
  rightTab = $state<RightTab>("changes");
  diff = $state<string>("");
  stagedDiff = $state<string>("");
  gitLog = $state<LogEntry[]>([]);
  gitStatus = $state<FileStatus[]>([]);
  profileId = $state<string>("");
  sidebarWidth = $state(280);
  sidebarCollapsed = $state(false);
  rightPanelCollapsed = $state(false);
  layout = $state<LayoutMode>("vertical");
  private _pollInterval: ReturnType<typeof setInterval> | null = null;
  private _pollActive = false;
  private _lastTerminalPerWorktree = new Map<string, string>();

  async addProject(path: string): Promise<ProjectInfo | null> {
    try {
      const info: ProjectInfo = await invoke("get_project_info", {
        projectPath: path,
      });
      const exists = this.projects.some((p) => p.path === info.path);
      if (!exists) {
        this.projects.push(info);
        this.saveProjects();
      }
      this.activeProject = info.path;
      if (info.worktrees.length > 0) {
        this.activeWorktree = info.worktrees[0].path;
      }
      return info;
    } catch (e) {
      console.error("Failed to add project:", e);
      return null;
    }
  }

  async refreshProject(path: string) {
    try {
      const info: ProjectInfo = await invoke("get_project_info", {
        projectPath: path,
      });
      const idx = this.projects.findIndex((p) => p.path === path);
      if (idx >= 0) {
        this.projects[idx] = info;
      }
    } catch (e) {
      console.error("Failed to refresh project:", e);
    }
  }

  removeProject(path: string) {
    // Close all terminals belonging to this project
    const project = this.projects.find((p) => p.path === path);
    if (project) {
      const wtPaths = new Set(project.worktrees.map((w) => w.path));
      const toRemove = this.terminals.filter((t) => wtPaths.has(t.worktreePath));
      for (const term of toRemove) {
        invoke("close_terminal", { id: term.id }).catch(() => {});
        this._lastTerminalPerWorktree.delete(term.worktreePath);
      }
      this.terminals = this.terminals.filter((t) => !wtPaths.has(t.worktreePath));
    }

    this.projects = this.projects.filter((p) => p.path !== path);

    if (this.activeProject === path) {
      const next = this.projects[0];
      if (next) {
        this.activeProject = next.path;
        this.activeWorktree = next.worktrees[0]?.path ?? "";
        this.activeTerminalId = this.activeWorktree
          ? (this.getTerminalsForWorktree(this.activeWorktree)[0]?.id ?? "")
          : "";
      } else {
        this.activeProject = "";
        this.activeWorktree = "";
        this.activeTerminalId = "";
      }
      this.refreshRightPanel();
    }

    this.saveProjects();
  }

  selectWorktree(projectPath: string, worktreePath: string) {
    // Save current terminal for the worktree we're leaving
    if (this.activeWorktree && this.activeTerminalId) {
      this._lastTerminalPerWorktree.set(this.activeWorktree, this.activeTerminalId);
    }

    this.activeProject = projectPath;
    this.activeWorktree = worktreePath;

    // Restore last terminal for the target worktree, or fall back to first
    const lastTermId = this._lastTerminalPerWorktree.get(worktreePath);
    const lastTerm = lastTermId ? this.terminals.find((t) => t.id === lastTermId) : null;
    if (lastTerm && lastTerm.worktreePath === worktreePath) {
      this.activeTerminalId = lastTerm.id;
    } else {
      const wtTerminals = this.getTerminalsForWorktree(worktreePath);
      this.activeTerminalId = wtTerminals[0]?.id ?? "";
    }
    this.refreshRightPanel();
  }

  async refreshRightPanel() {
    if (!this.activeWorktree) return;
    try {
      const [diff, stagedDiff, log, status] = await Promise.all([
        invoke("get_diff", { worktreePath: this.activeWorktree }) as Promise<string>,
        invoke("get_staged_diff", { worktreePath: this.activeWorktree }) as Promise<string>,
        invoke("get_git_log", { worktreePath: this.activeWorktree }) as Promise<LogEntry[]>,
        invoke("get_git_status", { worktreePath: this.activeWorktree }) as Promise<FileStatus[]>,
      ]);
      this.diff = diff;
      this.stagedDiff = stagedDiff;
      this.gitLog = log;
      this.gitStatus = status;
    } catch (e) {
      console.error("Failed to refresh panel:", e);
    }
  }

  async refreshAllProjects() {
    for (const project of this.projects) {
      await this.refreshProject(project.path);
    }
  }

  startPolling() {
    if (this._pollActive) return;
    this._pollActive = true;
    let pollCount = 0;
    this._pollInterval = setInterval(() => {
      if (this.activeWorktree && (this.rightTab === "changes" || this.rightTab === "timeline" || this.rightTab === "info")) {
        this.refreshRightPanel();
      }
      // Refresh worktree lists every ~10 seconds (every 5th poll)
      pollCount++;
      if (pollCount % 5 === 0) {
        this.refreshAllProjects();
      }
    }, 2000);
  }

  stopPolling() {
    this._pollActive = false;
    if (this._pollInterval) {
      clearInterval(this._pollInterval);
      this._pollInterval = null;
    }
  }

  addTerminal(terminal: TerminalInfo) {
    this.terminals.push(terminal);
    this.setActiveTerminal(terminal.id);
  }

  setActiveTerminal(id: string) {
    this.activeTerminalId = id;
    const term = this.terminals.find((t) => t.id === id);
    if (term) {
      this._lastTerminalPerWorktree.set(term.worktreePath, id);
      const ownerProject = this.projects.find((p) =>
        p.worktrees.some((w) => w.path === term.worktreePath)
      );
      if (ownerProject) {
        this.activeProject = ownerProject.path;
        this.activeWorktree = term.worktreePath;
      }
    }
  }

  removeTerminal(id: string) {
    const removed = this.terminals.find((t) => t.id === id);
    this.terminals = this.terminals.filter((t) => t.id !== id);
    // Clean up last-terminal tracking
    if (removed && this._lastTerminalPerWorktree.get(removed.worktreePath) === id) {
      this._lastTerminalPerWorktree.delete(removed.worktreePath);
    }
    if (this.activeTerminalId === id) {
      const wtTerminals = this.activeWorktree
        ? this.getTerminalsForWorktree(this.activeWorktree)
        : [];
      this.activeTerminalId = wtTerminals[0]?.id ?? this.terminals[0]?.id ?? "";
    }
  }

  renameTerminal(id: string, name: string) {
    const term = this.terminals.find((t) => t.id === id);
    if (term) {
      term.name = name;
    }
  }

  reorderTerminals(fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;
    const item = this.terminals.splice(fromIndex, 1)[0];
    if (item) {
      this.terminals.splice(toIndex, 0, item);
    }
  }

  getTerminalsForWorktree(worktreePath: string): TerminalInfo[] {
    return this.terminals.filter((t) => t.worktreePath === worktreePath);
  }

  getTerminalsForProject(projectPath: string): TerminalInfo[] {
    const project = this.projects.find((p) => p.path === projectPath);
    if (!project) return [];
    const wtPaths = new Set(project.worktrees.map((w) => w.path));
    return this.terminals.filter((t) => wtPaths.has(t.worktreePath));
  }

  setLayout(mode: LayoutMode) {
    this.layout = mode;
    localStorage.setItem("agentic-ide-layout", mode);
    // Also persist to profile backend
    if (this.profileId) {
      updateProfileSettings(this.profileId, {
        layout: mode,
        sidebar_width: this.sidebarWidth,
      }).catch(console.error);
    }
  }

  saveProjects() {
    const paths = this.projects.map((p) => p.path);
    localStorage.setItem("agentic-ide-projects", JSON.stringify(paths));
  }

  async loadProjects() {
    const stored = localStorage.getItem("agentic-ide-projects");
    if (stored) {
      const paths: string[] = JSON.parse(stored);
      for (const path of paths) {
        await this.addProject(path);
      }
    }
  }

  async initializeWithProfile(profileId?: string) {
    await profileState.initialize(profileId);
    const profile = profileState.activeProfile;
    if (profile) {
      this.profileId = profile.id;
      this.layout = profile.settings.layout as LayoutMode;
      this.sidebarWidth = profile.settings.sidebar_width;
    } else {
      // Fallback to localStorage if profile system not available
      const stored = localStorage.getItem("agentic-ide-layout") as LayoutMode;
      if (stored) this.layout = stored;
    }
  }
}

export const appState = new AppState();
