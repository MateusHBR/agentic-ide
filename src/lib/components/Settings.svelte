<script lang="ts">
  import { onMount } from "svelte";
  import { appState } from "$lib/state.svelte";
  import type { LayoutMode } from "$lib/state.svelte";
  import { isEnabled, enable, disable } from "@tauri-apps/plugin-autostart";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();

  let autoStartEnabled = $state(false);
  let autoStartLoading = $state(false);

  onMount(async () => {
    try {
      autoStartEnabled = await isEnabled();
    } catch (e) {
      console.error("Failed to check autostart:", e);
    }
  });

  async function toggleAutoStart() {
    autoStartLoading = true;
    try {
      if (autoStartEnabled) {
        await disable();
      } else {
        await enable();
      }
      autoStartEnabled = await isEnabled();
    } catch (e) {
      console.error("Failed to toggle autostart:", e);
    }
    autoStartLoading = false;
  }

  const shortcuts = [
    { keys: "⌘ N", description: "New terminal in active worktree" },
    { keys: "⌘ W", description: "Open worktree switcher" },
    { keys: "⌘ 1–9", description: "Switch to terminal by index (scoped to active worktree)" },
    { keys: "⌘ B", description: "Toggle sidebar" },
    { keys: "⌘ J", description: "Toggle right / bottom panel" },
    { keys: "⌘ ,", description: "Open settings" },
    { keys: "Esc", description: "Close overlay / modal" },
  ];

  const sections = [
    {
      title: "Keyboard Shortcuts",
      items: shortcuts,
    },
  ];

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="settings" onclick={(e) => e.stopPropagation()}>
    <div class="settings-header">
      <h2>Settings</h2>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="close-btn" onclick={onClose}>✕</span>
    </div>

    <div class="settings-content">
      {#each sections as section}
        <div class="section">
          <h3 class="section-title">{section.title}</h3>
          <div class="shortcut-list">
            {#each section.items as item}
              <div class="shortcut-row">
                <div class="shortcut-keys">
                  {#each item.keys.split(" ") as key}
                    {#if key === "+" || key === "–"}
                      <span class="key-sep">{key}</span>
                    {:else}
                      <kbd>{key}</kbd>
                    {/if}
                  {/each}
                </div>
                <span class="shortcut-desc">{item.description}</span>
              </div>
            {/each}
          </div>
        </div>
      {/each}

      <div class="section">
        <h3 class="section-title">Layout</h3>
        <div class="layout-options">
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="layout-option"
            class:active={appState.layout === "vertical"}
            onclick={() => appState.setLayout("vertical")}
          >
            <div class="layout-preview vertical-preview">
              <div class="lp-sidebar"></div>
              <div class="lp-center"></div>
              <div class="lp-right"></div>
            </div>
            <span class="layout-label">Vertical</span>
          </div>
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="layout-option"
            class:active={appState.layout === "horizontal"}
            onclick={() => appState.setLayout("horizontal")}
          >
            <div class="layout-preview horizontal-preview">
              <div class="lp-top"></div>
              <div class="lp-bottom"></div>
            </div>
            <span class="layout-label">Horizontal</span>
          </div>
        </div>
      </div>

      <div class="section">
        <h3 class="section-title">General</h3>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Launch at startup</span>
            <span class="setting-desc">Automatically start Agentic IDE when you log in</span>
          </div>
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="toggle-switch"
            class:on={autoStartEnabled}
            class:loading={autoStartLoading}
            onclick={toggleAutoStart}
          >
            <div class="toggle-knob"></div>
          </div>
        </div>
      </div>

      <div class="section">
        <h3 class="section-title">About</h3>
        <div class="about-row">
          <span class="about-label">Application</span>
          <span class="about-value">Agentic IDE</span>
        </div>
        <div class="about-row">
          <span class="about-label">Version</span>
          <span class="about-value">0.1.0</span>
        </div>
      </div>
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
    align-items: center;
    justify-content: center;
  }

  .settings {
    width: 540px;
    max-height: 80vh;
    background: #1c1c1e;
    border: 1px solid #3a3a3c;
    border-radius: 12px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid #2a2a2c;
  }

  .settings-header h2 {
    font-size: 16px;
    font-weight: 600;
    color: #e6edf3;
    margin: 0;
  }

  .close-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    cursor: pointer;
    color: #8b949e;
    font-size: 14px;
    transition: all 0.15s;
  }

  .close-btn:hover {
    background: #2a2a2c;
    color: #e6edf3;
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px 20px;
  }

  .section {
    margin-bottom: 24px;
  }

  .section:last-child {
    margin-bottom: 0;
  }

  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #8b949e;
    margin: 0 0 12px 0;
  }

  .shortcut-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-radius: 6px;
    transition: background 0.15s;
  }

  .shortcut-row:hover {
    background: #2a2a2c;
  }

  .shortcut-keys {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .shortcut-keys kbd {
    font-family: 'SF Mono', Menlo, monospace;
    font-size: 12px;
    color: #e6edf3;
    background: #2a2a2c;
    border: 1px solid #3a3a3c;
    padding: 3px 8px;
    border-radius: 5px;
    min-width: 24px;
    text-align: center;
  }

  .key-sep {
    color: #636366;
    font-size: 12px;
  }

  .shortcut-desc {
    font-size: 13px;
    color: #8b949e;
  }

  .layout-options {
    display: flex;
    gap: 12px;
  }

  .layout-option {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border: 2px solid #3a3a3c;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .layout-option:hover {
    border-color: #58a6ff;
    background: rgba(88, 166, 255, 0.05);
  }

  .layout-option.active {
    border-color: #30d158;
    background: rgba(48, 209, 88, 0.08);
  }

  .layout-label {
    font-size: 12px;
    color: #8b949e;
  }

  .layout-option.active .layout-label {
    color: #30d158;
    font-weight: 600;
  }

  .layout-preview {
    width: 100%;
    height: 48px;
    border-radius: 4px;
    overflow: hidden;
    display: flex;
    gap: 2px;
    background: #0d1117;
  }

  .vertical-preview {
    flex-direction: row;
  }

  .lp-sidebar {
    width: 20%;
    background: #2a2a2c;
    border-radius: 2px;
  }

  .lp-center {
    flex: 1;
    background: #1a1a2e;
    border-radius: 2px;
  }

  .lp-right {
    width: 25%;
    background: #161b22;
    border-radius: 2px;
  }

  .horizontal-preview {
    flex-direction: column;
  }

  .lp-top {
    height: 55%;
    background: #1a1a2e;
    border-radius: 2px;
    display: flex;
    gap: 2px;
  }

  .lp-top::before {
    content: "";
    width: 20%;
    background: #2a2a2c;
    border-radius: 2px;
  }

  .lp-top::after {
    content: "";
    flex: 1;
    background: #1a1a2e;
    border-radius: 2px;
  }

  .lp-bottom {
    flex: 1;
    background: #161b22;
    border-radius: 2px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-radius: 6px;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .setting-label {
    font-size: 13px;
    color: #e6edf3;
    font-weight: 500;
  }

  .setting-desc {
    font-size: 11px;
    color: #636366;
  }

  .toggle-switch {
    width: 44px;
    height: 24px;
    border-radius: 12px;
    background: #3a3a3c;
    cursor: pointer;
    position: relative;
    transition: background 0.2s;
    flex-shrink: 0;
  }

  .toggle-switch.on {
    background: #30d158;
  }

  .toggle-switch.loading {
    opacity: 0.5;
    pointer-events: none;
  }

  .toggle-knob {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    position: absolute;
    top: 2px;
    left: 2px;
    transition: transform 0.2s;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }

  .toggle-switch.on .toggle-knob {
    transform: translateX(20px);
  }

  .about-row {
    display: flex;
    justify-content: space-between;
    padding: 6px 12px;
    font-size: 13px;
  }

  .about-label {
    color: #8b949e;
  }

  .about-value {
    color: #e6edf3;
  }
</style>
