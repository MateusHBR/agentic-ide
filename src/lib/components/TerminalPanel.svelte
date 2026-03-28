<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { appState } from "$lib/state.svelte";

  interface Props {
    terminalId: string;
    worktreePath: string;
  }
  let { terminalId, worktreePath }: Props = $props();

  let terminalEl: HTMLDivElement;
  let term: any;
  let fitAddon: any;
  let unlisten: UnlistenFn | null = null;
  let resizeObserver: ResizeObserver | null = null;

  onMount(async () => {
    const { Terminal } = await import("@xterm/xterm");
    const { FitAddon } = await import("@xterm/addon-fit");

    term = new Terminal({
      cursorBlink: true,
      fontSize: 13,
      fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace",
      lineHeight: 1.3,
      theme: {
        background: "#0d1117",
        foreground: "#e6edf3",
        cursor: "#58a6ff",
        cursorAccent: "#0d1117",
        selectionBackground: "#264f78",
        selectionForeground: "#ffffff",
        black: "#484f58",
        red: "#ff7b72",
        green: "#3fb950",
        yellow: "#d29922",
        blue: "#58a6ff",
        magenta: "#bc8cff",
        cyan: "#39c5cf",
        white: "#b1bac4",
        brightBlack: "#6e7681",
        brightRed: "#ffa198",
        brightGreen: "#56d364",
        brightYellow: "#e3b341",
        brightBlue: "#79c0ff",
        brightMagenta: "#d2a8ff",
        brightCyan: "#56d4dd",
        brightWhite: "#f0f6fc",
      },
      scrollback: 10000,
      allowProposedApi: true,
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(terminalEl);

    // Small delay to ensure DOM is ready
    requestAnimationFrame(() => {
      fitAddon.fit();
      // Notify backend of initial size
      invoke("resize_terminal", {
        id: terminalId,
        rows: term.rows,
        cols: term.cols,
      }).catch(console.error);
    });

    // Listen for terminal output from backend
    unlisten = await listen("terminal-output", (event: any) => {
      const payload = event.payload;
      if (payload.id === terminalId && term) {
        term.write(payload.data);
      }
    });

    // Send user input to backend
    term.onData((data: string) => {
      invoke("write_terminal", { id: terminalId, data }).catch(console.error);
    });

    // Handle resize
    resizeObserver = new ResizeObserver(() => {
      if (fitAddon && term) {
        fitAddon.fit();
        invoke("resize_terminal", {
          id: terminalId,
          rows: term.rows,
          cols: term.cols,
        }).catch(console.error);
      }
    });
    resizeObserver.observe(terminalEl);

    if (appState.activeTerminalId === terminalId) {
      requestAnimationFrame(() => term.focus());
    }
  });

  $effect(() => {
    if (appState.activeTerminalId === terminalId && term) {
      requestAnimationFrame(() => term.focus());
    }
  });

  onDestroy(() => {
    unlisten?.();
    resizeObserver?.disconnect();
    term?.dispose();
  });
</script>

<div class="terminal-container">
  <div class="terminal-wrapper" bind:this={terminalEl}></div>
</div>

<style>
  .terminal-container {
    width: 100%;
    height: 100%;
    background: #0d1117;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .terminal-wrapper {
    flex: 1;
    padding: 4px;
    padding-top: 20px;
    overflow: hidden;
  }

  :global(.terminal-wrapper .xterm) {
    height: 100%;
  }

  :global(.terminal-wrapper .xterm-viewport) {
    overflow-y: auto !important;
  }

  :global(.terminal-wrapper .xterm-screen) {
    height: 100% !important;
  }
</style>
