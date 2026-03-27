# Agentic IDE - Agent Development Guide

## Project Overview

A desktop application for managing git worktrees with embedded Claude Code terminals. Built with **Tauri 2** (Rust backend) + **SvelteKit** (Svelte 5 frontend). Three-panel layout: sidebar (projects/worktrees), center (terminal), right (diffs/timeline/agent info).

## Tech Stack

- **Backend:** Rust, Tauri 2, `portable-pty` (PTY terminals), `uuid`
- **Frontend:** Svelte 5 (runes: `$state`, `$derived`, `$effect`), SvelteKit, TypeScript
- **Terminal:** xterm.js (`@xterm/xterm`, `@xterm/addon-fit`)
- **Plugins:** `tauri-plugin-dialog` (native file picker), `tauri-plugin-opener`
- **Package manager:** pnpm
- **Build:** Vite 6, `@sveltejs/adapter-static` (SPA mode, no SSR)

## Architecture

### Rust Backend (`src-tauri/src/`)

| File | Purpose |
|---|---|
| `main.rs` | Entry point, calls `lib::run()` |
| `lib.rs` | Tauri app setup, all `#[tauri::command]` handlers, state management |
| `terminal.rs` | PTY management via `portable-pty`. `TerminalManager` creates/writes/resizes/closes terminals. Reader thread emits `terminal-output` and `terminal-exit` Tauri events. |
| `worktree.rs` | Git operations: `list_worktrees` (parses `--porcelain`), `get_diff`, `get_staged_diff`, `get_file_diff` (configurable context lines), `get_log`, `get_status`, `stage_file`, `unstage_file`, `add_worktree`, `remove_worktree`, `get_project_info` |

**Tauri commands exposed:** `create_terminal`, `write_terminal`, `resize_terminal`, `close_terminal`, `list_worktrees`, `get_project_info`, `add_worktree`, `remove_worktree`, `get_diff`, `get_staged_diff`, `get_file_diff`, `get_git_log`, `get_git_status`, `stage_file`, `unstage_file`

**State:** `Mutex<TerminalManager>` managed by Tauri.

### Svelte Frontend (`src/`)

| File | Purpose |
|---|---|
| `app.css` | Global styles, dark theme, scrollbar, user-select disabled globally (re-enabled for terminal/diffs) |
| `app.html` | HTML shell |
| `routes/+page.svelte` | Main three-panel layout, resize handles, keyboard shortcuts (`Cmd+N/W/,/1-9`), terminal lifecycle, modal orchestration (worktree switcher, settings) |
| `routes/+layout.svelte` | Imports global CSS |
| `routes/+layout.ts` | `ssr = false` for Tauri SPA |
| `lib/state.svelte.ts` | `AppState` class — central state. Manages projects, terminals, worktrees, diffs, git status/log. Polling (2s). Persists project paths to localStorage. Tracks last active terminal per worktree (`_lastTerminalPerWorktree`). All scoping is per-worktree (not per-project). |
| `lib/components/Sidebar.svelte` | Left panel: project list, worktree tree with green active indicator (border + dot + text), terminals grouped under each worktree, rename (dblclick), drag-reorder, "Add Project" (native dialog), "Settings" button. |
| `lib/components/TerminalPanel.svelte` | xterm.js terminal. Connects to Rust PTY via Tauri events. Auto-focus on switch via `$effect`. ResizeObserver for fit. |
| `lib/components/RightPanel.svelte` | Tabs: Info, Timeline, Agent, Changes. Changes tab: separate unstaged/staged sections (`git diff` vs `git diff --cached`), collapsible sections and files, stage/unstage checkboxes, "Stage All" button, line numbers, expandable context (GitHub-style `↑`/`⇕`/`↓` arrows via `get_file_diff`). |
| `lib/components/WorktreeSwitcher.svelte` | `Cmd+W` modal: lists all worktrees across projects, filterable search, arrow key navigation, Enter to select, Escape to close. Shows current worktree with green dot + "current" badge. |
| `lib/components/Settings.svelte` | `Cmd+,` or sidebar button: shows all keyboard shortcuts with styled kbd keys, app version info. |

### Data Flow

1. **Terminal I/O:** Frontend `invoke("write_terminal")` -> Rust PTY write. Rust reader thread -> `emit("terminal-output")` -> Frontend `listen()` -> xterm.js `write()`.
2. **Terminal exit:** PTY reader EOF -> `emit("terminal-exit")` -> `+page.svelte` listener removes terminal from state and calls `close_terminal`.
3. **Git data:** Polling every 2s calls `get_diff`, `get_staged_diff`, `get_git_log`, `get_git_status` and updates reactive state. Right panel re-renders via `$derived`.
4. **Projects:** Stored as path list in localStorage. On load, each path is re-fetched via `get_project_info`.
5. **Worktree switching:** `selectWorktree()` saves current terminal to `_lastTerminalPerWorktree`, restores last terminal for target worktree (or falls back to first). `setActiveTerminal()` syncs `activeProject` and `activeWorktree` to match the terminal's owner.

### Scoping Model

Everything is scoped by **worktree**, not by project:
- **Keyboard shortcuts** (`Cmd+1-9`): switch between terminals of the active worktree only
- **Bottom tab bar**: shows only the active worktree's terminals with `⌘N` hints
- **Terminal naming**: per-worktree count (each worktree starts at "Terminal 1")
- **Last active terminal**: remembered per-worktree, restored on switch back
- **Changes/diffs**: fetched for the active worktree path

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Cmd + N` | New terminal in active worktree |
| `Cmd + 1-9` | Switch to terminal by index (scoped to active worktree) |
| `Cmd + W` | Open/close worktree switcher |
| `Cmd + ,` | Open/close settings |
| `Escape` | Close any open modal |

Shortcuts are registered on `window` with `capture: true` so they fire before xterm.js intercepts keyboard input. When a modal is open, terminal shortcuts (`N`, `1-9`) are suppressed.

## Key Patterns

- **Svelte 5 runes:** Use `$state` for reactive variables, `$derived` for computed values, `$effect` for side effects. No stores or `$:` syntax.
- **Event modifiers:** Svelte 5 does NOT support `onclick|stopPropagation`. Use `(e) => { e.stopPropagation(); ... }` instead.
- **CSS scoping:** `@import` cannot be used inside `<style>` blocks in Svelte components. Use `app.css` for global imports. `:global()` for targeting child component classes.
- **Read-only `$derived`:** Cannot mutate `$derived` values. To override derived data (e.g., expanded diff context), use a separate `$state` map for overrides and merge in a new `$derived`.
- **User-select:** Globally disabled via `app.css`. Explicitly re-enabled for terminal (`.xterm`), diff content, file names, commit hashes.
- **Window dragging:** Overlay titlebar (38px fixed) with `data-tauri-drag-region` + `getCurrentWindow().startDragging()` on mousedown.
- **Tauri permissions:** Defined in `src-tauri/capabilities/default.json`. Must include permissions for any new plugins or window operations.
- **Panel resizing:** Sidebar and right panel draggable from 180px to 50% of window width. Center panel has `min-width: 300px`.
- **Terminal auto-focus:** `$effect` in `TerminalPanel.svelte` watches `appState.activeTerminalId` and calls `term.focus()` via `requestAnimationFrame`.

## Build & Run

```bash
pnpm install          # Install frontend deps
pnpm build            # Build frontend (writes to build/)
pnpm tauri dev        # Dev mode (hot reload frontend + Rust backend)
pnpm tauri build      # Production build

# Rust-only check:
cd src-tauri && cargo check
```

**Dev server:** Port 1420 (Vite). Tauri loads from `http://localhost:1420` in dev mode.

## Adding New Features

### New Tauri command:
1. Add function in `worktree.rs` or `terminal.rs`
2. Add `#[tauri::command]` wrapper in `lib.rs`
3. Register in `generate_handler![]` macro in `lib.rs`
4. Call from frontend via `invoke("command_name", { params })`

### New Tauri plugin:
1. Add to `Cargo.toml` dependencies
2. Add npm package to `package.json`
3. Register plugin in `lib.rs` with `.plugin()`
4. Add permission to `src-tauri/capabilities/default.json`

### New Svelte component:
1. Create in `src/lib/components/`
2. Use Svelte 5 runes (`$state`, `$derived`, `$effect`)
3. Import and use in parent component

### New keyboard shortcut:
1. Add handler in `handleKeydown()` in `+page.svelte` (uses `capture: true`)
2. Guard with `if (showWorktreeSwitcher || showSettings) return;` to suppress when modals are open
3. Add entry to `Settings.svelte` shortcuts list
4. Update this file

### New state:
1. Add `$state` field to `AppState` class in `state.svelte.ts`
2. Add methods for mutations
3. Access via `appState.fieldName` in components

## Configuration

| File | What it configures |
|---|---|
| `src-tauri/tauri.conf.json` | Window size (1440x900), title, titlebar style (overlay), bundle settings |
| `src-tauri/capabilities/default.json` | Tauri permissions: `core:default`, `core:window:allow-start-dragging`, `core:window:allow-start-resize-dragging`, `opener:default`, `dialog:default` |
| `svelte.config.js` | Static adapter, SPA fallback, vitePreprocess |
| `vite.config.js` | Port 1420, HMR, ignores src-tauri/ from watch |
| `tsconfig.json` | Strict mode, bundler module resolution |

## Releasing a New Version

### Step-by-step process

1. **Bump version** in all 4 files:
   - `src-tauri/tauri.conf.json` → `"version": "X.Y.Z"`
   - `src-tauri/Cargo.toml` → `version = "X.Y.Z"`
   - `package.json` → `"version": "X.Y.Z"`
   - `src/lib/components/Settings.svelte` → the about value string

2. **Commit and push**:
   ```bash
   git add -A && git commit -m "Bump version to X.Y.Z" && git push origin main
   ```

3. **Build with signing** (generates DMG + updater artifacts):
   ```bash
   rm -rf src-tauri/target/release/bundle/
   TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/agentic-ide.key)" \
   TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" \
   pnpm tauri build
   ```
   This produces:
   - `src-tauri/target/release/bundle/dmg/Agentic IDE_X.Y.Z_aarch64.dmg` — installer
   - `src-tauri/target/release/bundle/macos/Agentic IDE.app.tar.gz` — updater artifact
   - `src-tauri/target/release/bundle/macos/Agentic IDE.app.tar.gz.sig` — updater signature

4. **Generate `latest.json`** for the auto-updater:
   ```bash
   BUNDLE="src-tauri/target/release/bundle"
   SIG=$(cat "$BUNDLE/macos/Agentic IDE.app.tar.gz.sig")
   DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
   cat > /tmp/latest.json << EOF
   {
     "version": "X.Y.Z",
     "notes": "Release notes here",
     "pub_date": "$DATE",
     "platforms": {
       "darwin-aarch64": {
         "signature": "$SIG",
         "url": "https://github.com/MateusHBR/agentic-ide/releases/download/vX.Y.Z/Agentic.IDE.app.tar.gz"
       }
     }
   }
   EOF
   ```

5. **Tag and push**:
   ```bash
   git tag vX.Y.Z && git push origin vX.Y.Z
   ```

6. **Create GitHub Release** with all 4 artifacts:
   ```bash
   BUNDLE="src-tauri/target/release/bundle"
   gh release create vX.Y.Z \
     "$BUNDLE/dmg/Agentic IDE_X.Y.Z_aarch64.dmg" \
     "$BUNDLE/macos/Agentic IDE.app.tar.gz#Agentic.IDE.app.tar.gz" \
     "$BUNDLE/macos/Agentic IDE.app.tar.gz.sig#Agentic.IDE.app.tar.gz.sig" \
     "/tmp/latest.json" \
     --title "Agentic IDE vX.Y.Z" \
     --notes "Release notes here"
   ```

### How the auto-updater works

- The app checks `https://github.com/MateusHBR/agentic-ide/releases/latest/download/latest.json` on startup (3s delay) and every 30 minutes.
- `latest.json` contains the version, signature, and download URL for each platform.
- If a newer version is found, a floating toast appears in the bottom-right corner.
- The user can download, install, and restart from the toast.
- Updates are signed with a minisign keypair. The **public key** is in `tauri.conf.json` under `plugins.updater.pubkey`. The **private key** is at `~/.tauri/agentic-ide.key` (never commit this).
- The `tauri-plugin-process` plugin provides the `relaunch()` function for the restart button.

### Signing key location

- **Private key**: `~/.tauri/agentic-ide.key` — used to sign builds, set via `TAURI_SIGNING_PRIVATE_KEY` env var
- **Public key**: `~/.tauri/agentic-ide.key.pub` — embedded in `tauri.conf.json`
- **Password**: empty (no password)

### CI (GitHub Actions)

`.github/workflows/release.yml` triggers on tag push (`v*`). It builds for macOS aarch64 + x86_64 using `tauri-apps/tauri-action`. Requires `TAURI_SIGNING_PRIVATE_KEY` as a GitHub Actions secret.

### Important notes

- Always `rm -rf src-tauri/target/release/bundle/` before building — stale DMG files cause the bundler to fail.
- The `latest.json` URL in the release must use dots instead of spaces: `Agentic.IDE.app.tar.gz` (GitHub replaces spaces with dots in asset download URLs).
- The `createUpdaterArtifacts: true` flag in `tauri.conf.json` tells the bundler to produce `.tar.gz` + `.sig` files alongside the DMG.

## Gotchas

- `portable-pty` reader thread runs on a spawned OS thread, not tokio. Terminal output events are emitted from this thread.
- `$derived` values are **read-only** in Svelte 5. To override derived data (e.g., expanded diff context), use a separate `$state` map for overrides and merge in a new `$derived`.
- Tauri 2 webview on macOS needs explicit `-webkit-app-region: drag` AND `getCurrentWindow().startDragging()` for reliable window dragging.
- `git diff` (no args) diffs working tree vs index. `git diff --cached` diffs index vs HEAD. `git diff HEAD` diffs working tree vs HEAD. The app uses the first two for separate unstaged/staged sections.
- Scoping is per-**worktree**, not per-project. A single git repo can have multiple worktrees that each act as independent workspaces with their own terminals, diffs, and shortcuts.
- Terminal shortcuts must use `capture: true` on the event listener to fire before xterm.js intercepts keyboard input.
- The bottom tab bar and `Cmd+1-9` show `⌘N` shortcut hints next to each tab name for discoverability.
