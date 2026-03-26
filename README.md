# Agentic IDE

A desktop application for managing git worktrees with embedded terminals, built for developers who run multiple Claude Code sessions across different branches simultaneously.

## What it does

Agentic IDE gives you a single window to manage all your git worktrees. Each worktree gets its own set of terminals, and you can switch between them instantly with keyboard shortcuts. A built-in changes panel shows live diffs, staged/unstaged files, and git history -- all updating in real time as Claude Code (or you) modifies files.

## Screenshot

Three-panel layout: sidebar with projects and worktrees, center terminal, right panel with diffs and timeline.

## Features

### Worktree Management
- Add git projects via native macOS file picker
- View all worktrees per project in a tree structure
- Switch between worktrees with `Cmd+W` fuzzy finder
- Green active indicator on the selected worktree
- Persists projects across sessions (localStorage)

### Embedded Terminals
- Full PTY terminals powered by xterm.js + portable-pty
- Terminals are scoped per worktree (each worktree has its own set)
- Create terminals with `Cmd+N` or the sidebar button
- Switch terminals with `Cmd+1` through `Cmd+9`
- Rename terminals by double-clicking their name
- Drag-and-drop to reorder terminals
- Auto-focus on terminal switch -- start typing immediately
- Remembers last active terminal per worktree
- Terminals close cleanly when you type `exit`
- Bottom tab bar shows only the active worktree's terminals with shortcut hints

### Changes Panel (Right Side)
- **Unstaged Changes** -- live diff from `git diff` (working tree vs staging area)
- **Staged Changes** -- live diff from `git diff --cached` (staging area vs HEAD)
- Stage/unstage individual files with checkboxes
- "Stage All" button for bulk staging
- Collapsible file sections and collapsible section headers
- Line numbers on diffs (old and new)
- GitHub-style expand arrows to load more context lines per file
- Collapse back to default context after full expansion
- Auto-refreshes every 2 seconds via polling

### Right Panel Tabs
- **Info** -- worktree path, branch, HEAD hash, git status summary
- **Timeline** -- commit history with hashes, authors, relative timestamps
- **Agent** -- Claude Code session status
- **Changes** -- full diff viewer (described above)

### Layout
- Three resizable panels (sidebar, terminal, right panel)
- Switch between **vertical** (side-by-side) and **horizontal** (stacked) layouts in Settings
- Collapse sidebar or right panel to a 40px strip with one click
- Re-expand by clicking the collapsed strip
- All panels resize from 180px to 50% of window width
- Layout preference persisted to localStorage

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd + N` | New terminal in active worktree |
| `Cmd + 1-9` | Switch to terminal by index (scoped to active worktree) |
| `Cmd + W` | Open worktree switcher (fuzzy search, arrow keys, Enter) |
| `Cmd + B` | Toggle sidebar |
| `Cmd + J` | Toggle right/bottom panel |
| `Cmd + ,` | Open settings |
| `Esc` | Close any open modal |

### Settings
- Layout toggle (vertical/horizontal) with visual previews
- Full keyboard shortcuts reference
- Accessible via `Cmd+,` or the sidebar Settings button

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri 2 |
| Backend | Rust |
| Frontend framework | SvelteKit (Svelte 5 with runes) |
| Terminal emulator | xterm.js |
| PTY | portable-pty |
| Build tool | Vite 6 |
| Language | TypeScript + Rust |
| Package manager | pnpm |

## Project Structure

```
src/                              # Svelte frontend
  app.css                         # Global styles, dark theme
  app.html                        # HTML shell
  lib/
    state.svelte.ts               # Central app state (Svelte 5 runes)
    components/
      Sidebar.svelte              # Project/worktree tree, terminal list
      TerminalPanel.svelte        # xterm.js terminal wrapper
      RightPanel.svelte           # Info, Timeline, Agent, Changes tabs
      WorktreeSwitcher.svelte     # Cmd+W fuzzy worktree picker
      Settings.svelte             # Settings modal
  routes/
    +page.svelte                  # Main layout, shortcuts, panel orchestration
    +layout.svelte                # Global CSS import
    +layout.ts                    # SSR disabled for Tauri

src-tauri/                        # Rust backend
  src/
    main.rs                       # Entry point
    lib.rs                        # Tauri commands, app setup
    terminal.rs                   # PTY terminal management
    worktree.rs                   # Git worktree/diff/status operations
  Cargo.toml                      # Rust dependencies
  tauri.conf.json                 # Tauri window/app configuration
  capabilities/default.json       # Tauri permission grants
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/)
- Xcode Command Line Tools (macOS): `xcode-select --install`

### Development

```bash
pnpm install        # Install frontend dependencies
pnpm tauri dev      # Start dev mode (hot reload + Rust backend)
```

The app opens automatically. Vite dev server runs on port 1420.

### Production Build

```bash
pnpm tauri build    # Build macOS .app bundle
```

Output is in `src-tauri/target/release/bundle/`.

### Quick Checks

```bash
pnpm build                      # Build frontend only
cd src-tauri && cargo check     # Check Rust compilation
```

## How It Works

1. **Terminal I/O**: Frontend calls `invoke("write_terminal")` to send keystrokes to Rust. A background OS thread reads PTY output and emits `terminal-output` Tauri events back to xterm.js.

2. **Git Data**: Every 2 seconds, the frontend polls `git diff`, `git diff --cached`, `git log`, and `git status` for the active worktree. Results flow into Svelte 5 reactive state and the right panel re-renders automatically.

3. **Worktree Scoping**: Everything is scoped by worktree -- terminals, shortcuts, tab bar, diffs, and the last-active-terminal memory. A single git repo with multiple worktrees shows each as an independent workspace.

4. **Persistence**: Project paths are saved to localStorage and restored on launch. Layout preference (vertical/horizontal) is also persisted.

## Recommended IDE Setup

[Zed](https://zed.dev/), [VS Code](https://code.visualstudio.com/), or any editor with:
- Svelte language support
- Rust Analyzer
- Tauri extension (VS Code)

## License

MIT
