<script lang="ts">
  interface Props {
    onAddProject: () => void;
  }
  let { onAddProject }: Props = $props();
</script>

<div class="onboarding">
  <div class="onboarding-content">
    <div class="logo-container">
      <div class="logo-hex">
        <svg width="72" height="72" viewBox="0 0 72 72" fill="none">
          <path d="M36 4L64 20V52L36 68L8 52V20L36 4Z" stroke="#58a6ff" stroke-width="2" fill="none" opacity="0.6" />
          <path d="M36 12L56 24V48L36 60L16 48V24L36 12Z" stroke="#58a6ff" stroke-width="1.5" fill="none" opacity="0.3" />
          <circle cx="36" cy="36" r="6" fill="#58a6ff" opacity="0.8" />
        </svg>
      </div>
      <h1 class="title">Agentic IDE</h1>
      <p class="tagline">Manage git worktrees with multiple terminals, side by side.</p>
    </div>

    <!-- svelte-ignore a11y_autofocus -->
    <button class="cta-button" onclick={onAddProject} autofocus>
      <span class="cta-icon">+</span>
      Open a Project
    </button>
    <p class="cta-hint">Select a folder with a Git repository to get started</p>

    <div class="shortcuts">
      <div class="shortcut-grid">
        <div class="shortcut">
          <kbd>&#8984;N</kbd>
          <span>New Terminal</span>
        </div>
        <div class="shortcut">
          <kbd>&#8984;W</kbd>
          <span>Switch Worktree</span>
        </div>
        <div class="shortcut">
          <kbd>&#8984;B</kbd>
          <span>Toggle Sidebar</span>
        </div>
        <div class="shortcut">
          <kbd>&#8984;J</kbd>
          <span>Toggle Panel</span>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .onboarding {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #0d1117;
    width: 100%;
    height: 100%;
  }

  .onboarding-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 28px;
    max-width: 420px;
    padding: 40px;
  }

  .logo-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }

  .logo-hex {
    opacity: 0.9;
    margin-bottom: 4px;
  }

  .title {
    font-size: 28px;
    font-weight: 700;
    color: #e6edf3;
    letter-spacing: -0.5px;
  }

  .tagline {
    font-size: 14px;
    color: #8b949e;
    text-align: center;
    line-height: 1.5;
  }

  .cta-button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 32px;
    background: #238636;
    border: 1px solid #2ea043;
    border-radius: 8px;
    color: #fff;
    font-size: 15px;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    transition: background 0.15s, box-shadow 0.15s;
  }

  .cta-button:hover {
    background: #2ea043;
    box-shadow: 0 0 16px rgba(46, 160, 67, 0.3);
  }

  .cta-button:focus-visible {
    outline: 2px solid #58a6ff;
    outline-offset: 2px;
  }

  .cta-icon {
    font-size: 18px;
    font-weight: 400;
  }

  .cta-hint {
    font-size: 12px;
    color: #484f58;
    margin-top: -16px;
  }

  .shortcuts {
    margin-top: 12px;
    padding-top: 20px;
    border-top: 1px solid #21262d;
    width: 100%;
  }

  .shortcut-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .shortcut {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 12px;
    color: #8b949e;
  }

  .shortcut kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 36px;
    padding: 3px 8px;
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 5px;
    font-family: 'SF Mono', Menlo, monospace;
    font-size: 11px;
    color: #8b949e;
  }
</style>
