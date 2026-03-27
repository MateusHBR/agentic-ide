<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";

  type Status = "hidden" | "available" | "downloading" | "ready" | "error";

  let status = $state<Status>("hidden");
  let version = $state("");
  let errorMsg = $state("");
  let dismissed = $state(false);
  let update = $state<Update | null>(null);
  let checkInterval: ReturnType<typeof setInterval> | null = null;

  const CHECK_INTERVAL = 30 * 60 * 1000; // 30 minutes

  async function checkForUpdate() {
    try {
      const result = await check();
      if (result) {
        update = result;
        version = result.version;
        status = "available";
        dismissed = false;
      }
    } catch (_) {
      // Silently ignore — background check
    }
  }

  async function downloadAndInstall() {
    if (!update) return;
    status = "downloading";
    try {
      await update.downloadAndInstall();
      status = "ready";
    } catch (e: any) {
      status = "error";
      errorMsg = e?.message ?? String(e);
    }
  }

  async function restart() {
    await relaunch();
  }

  function dismiss() {
    dismissed = true;
  }

  onMount(() => {
    // Check on startup after a short delay
    setTimeout(checkForUpdate, 3000);
    // Check every 30 minutes
    checkInterval = setInterval(checkForUpdate, CHECK_INTERVAL);
  });

  onDestroy(() => {
    if (checkInterval) clearInterval(checkInterval);
  });

  let visible = $derived(status !== "hidden" && !dismissed);
</script>

{#if visible}
  <div class="update-toast">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <span class="toast-close" onclick={dismiss}>✕</span>

    {#if status === "available"}
      <div class="toast-icon">↑</div>
      <div class="toast-body">
        <span class="toast-title">Update Available</span>
        <span class="toast-version">v{version} is ready to download</span>
      </div>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <button class="toast-btn primary" onclick={downloadAndInstall}>Update</button>

    {:else if status === "downloading"}
      <div class="toast-icon spin">↻</div>
      <div class="toast-body">
        <span class="toast-title">Downloading...</span>
        <span class="toast-version">Installing v{version}</span>
      </div>

    {:else if status === "ready"}
      <div class="toast-icon done">✓</div>
      <div class="toast-body">
        <span class="toast-title">Update Installed</span>
        <span class="toast-version">Restart to apply v{version}</span>
      </div>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <button class="toast-btn restart" onclick={restart}>Restart</button>

    {:else if status === "error"}
      <div class="toast-icon error">!</div>
      <div class="toast-body">
        <span class="toast-title">Update Failed</span>
        <span class="toast-version">{errorMsg}</span>
      </div>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <button class="toast-btn" onclick={downloadAndInstall}>Retry</button>
    {/if}
  </div>
{/if}

<style>
  .update-toast {
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 300;
    display: flex;
    align-items: center;
    gap: 12px;
    background: #1c1c1e;
    border: 1px solid #3a3a3c;
    border-radius: 10px;
    padding: 12px 16px;
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.5);
    min-width: 280px;
    max-width: 380px;
    animation: slideIn 0.3s ease-out;
  }

  @keyframes slideIn {
    from {
      transform: translateY(20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  .toast-close {
    position: absolute;
    top: 6px;
    right: 8px;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    cursor: pointer;
    color: #636366;
    font-size: 11px;
    transition: all 0.15s;
  }

  .toast-close:hover {
    color: #e6edf3;
    background: #2a2a2c;
  }

  .toast-icon {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    font-weight: 700;
    flex-shrink: 0;
    background: rgba(88, 166, 255, 0.15);
    color: #58a6ff;
  }

  .toast-icon.spin {
    animation: spin 1s linear infinite;
  }

  .toast-icon.done {
    background: rgba(48, 209, 88, 0.15);
    color: #30d158;
  }

  .toast-icon.error {
    background: rgba(255, 123, 114, 0.15);
    color: #ff7b72;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .toast-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .toast-title {
    font-size: 13px;
    font-weight: 600;
    color: #e6edf3;
  }

  .toast-version {
    font-size: 11px;
    color: #8b949e;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .toast-btn {
    padding: 6px 14px;
    border: 1px solid #30363d;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
    transition: background 0.15s;
    flex-shrink: 0;
    background: #21262d;
    color: #e6edf3;
  }

  .toast-btn:hover {
    background: #30363d;
  }

  .toast-btn.primary {
    background: #238636;
    border-color: #2ea043;
    color: #fff;
  }

  .toast-btn.primary:hover {
    background: #2ea043;
  }

  .toast-btn.restart {
    background: #1f6feb;
    border-color: #388bfd;
    color: #fff;
  }

  .toast-btn.restart:hover {
    background: #388bfd;
  }
</style>
