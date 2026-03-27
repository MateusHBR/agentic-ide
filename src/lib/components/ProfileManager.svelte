<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { profileState } from "$lib/profiles.svelte";
  import type { Profile } from "$lib/profiles.svelte";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();

  let newProfileName = $state("");
  let newProfileColor = $state("#58a6ff");
  let isCreating = $state(false);
  let editingId = $state<string | null>(null);
  let editName = $state("");
  let editColor = $state("");

  const presetColors = [
    "#58a6ff", "#3fb950", "#d29922", "#ff7b72",
    "#bc8cff", "#39c5cf", "#f778ba", "#ffa657",
  ];

  async function handleCreate() {
    if (!newProfileName.trim()) return;
    isCreating = true;
    await profileState.createNewProfile(newProfileName.trim(), newProfileColor);
    newProfileName = "";
    newProfileColor = "#58a6ff";
    isCreating = false;
  }

  async function handleDelete(id: string) {
    await profileState.deleteExistingProfile(id);
  }

  async function handleSetDefault(id: string) {
    await profileState.setDefault(id);
  }

  async function handleOpenInNewWindow(profile: Profile) {
    try {
      await invoke("open_profile_window", {
        profileId: profile.id,
        profileName: profile.name,
      });
    } catch (e) {
      console.error("Failed to open profile window:", e);
    }
  }

  let editInputEl = $state<HTMLInputElement | null>(null);

  function startEdit(profile: Profile) {
    editingId = profile.id;
    editName = profile.name;
    editColor = profile.color;
    requestAnimationFrame(() => editInputEl?.focus());
  }

  function cancelEdit() {
    editingId = null;
  }

  async function commitEdit() {
    if (editingId && editName.trim()) {
      try {
        await invoke("update_profile", {
          id: editingId,
          name: editName.trim(),
          color: editColor,
        });
        await profileState.loadProfiles();
      } catch (e) {
        console.error("Failed to update profile:", e);
      }
    }
    editingId = null;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      if (editingId) {
        editingId = null;
      } else {
        onClose();
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h2>Profiles</h2>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span class="close-btn" onclick={onClose}>&#10005;</span>
    </div>

    <div class="modal-content">
      <div class="section">
        <h3 class="section-title">Your Profiles</h3>
        <div class="profile-list">
          {#each profileState.profiles as profile (profile.id)}
            <div class="profile-item">
              <div class="profile-info">
                <span class="profile-dot" style="background: {profile.color}"></span>
                {#if editingId === profile.id}
                  <div class="edit-form">
                    <input
                      class="edit-input"
                      type="text"
                      bind:this={editInputEl}
                      bind:value={editName}
                      onkeydown={(e) => {
                        if (e.key === "Enter") commitEdit();
                        if (e.key === "Escape") cancelEdit();
                      }}
                    />
                    <div class="edit-colors">
                      {#each presetColors as c}
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <span
                          class="color-swatch small"
                          class:selected={editColor === c}
                          style="background: {c}"
                          onclick={() => (editColor = c)}
                        ></span>
                      {/each}
                    </div>
                    <div class="edit-actions">
                      <button class="edit-btn cancel" onclick={cancelEdit}>Cancel</button>
                      <button class="edit-btn save" onclick={commitEdit} disabled={!editName.trim()}>Save</button>
                    </div>
                  </div>
                {:else}
                  <span class="profile-name">{profile.name}</span>
                  {#if profile.is_default}
                    <span class="default-badge">Default</span>
                  {/if}
                {/if}
              </div>
              {#if editingId !== profile.id}
                <div class="profile-actions">
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <span class="action-btn" onclick={() => startEdit(profile)} title="Edit">
                    &#9998;
                  </span>
                  {#if !profile.is_default}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <span class="action-btn" onclick={() => handleSetDefault(profile.id)} title="Set as default">
                      &#9733;
                    </span>
                  {/if}
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <span class="action-btn" onclick={() => handleOpenInNewWindow(profile)} title="Open in new window">
                    &#10697;
                  </span>
                  {#if !profile.is_default}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <span class="action-btn danger" onclick={() => handleDelete(profile.id)} title="Delete">
                      &#10005;
                    </span>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
          {#if profileState.profiles.length === 0}
            <div class="empty">No profiles yet</div>
          {/if}
        </div>
      </div>

      <div class="section">
        <h3 class="section-title">Create New Profile</h3>
        <div class="create-form">
          <input
            class="create-input"
            type="text"
            placeholder="Profile name..."
            bind:value={newProfileName}
            onkeydown={(e) => { if (e.key === "Enter") handleCreate(); }}
          />
          <div class="color-swatches">
            {#each presetColors as c}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <span
                class="color-swatch"
                class:selected={newProfileColor === c}
                style="background: {c}"
                onclick={() => (newProfileColor = c)}
              ></span>
            {/each}
          </div>
          <button class="create-btn" onclick={handleCreate} disabled={isCreating || !newProfileName.trim()}>
            {isCreating ? "Creating..." : "Create Profile"}
          </button>
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

  .modal {
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

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid #2a2a2c;
  }

  .modal-header h2 {
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

  .modal-content {
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

  .profile-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .profile-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-radius: 6px;
    transition: background 0.15s;
  }

  .profile-item:hover {
    background: #2a2a2c;
  }

  .profile-info {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    min-width: 0;
  }

  .profile-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .profile-name {
    font-size: 13px;
    color: #e6edf3;
    font-weight: 500;
  }

  .default-badge {
    font-size: 10px;
    color: #30d158;
    background: rgba(48, 209, 88, 0.15);
    padding: 2px 8px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .profile-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .profile-item:hover .profile-actions {
    opacity: 1;
  }

  .action-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    cursor: pointer;
    color: #8b949e;
    font-size: 13px;
    transition: all 0.15s;
  }

  .action-btn:hover {
    background: #3a3a3c;
    color: #e6edf3;
  }

  .action-btn.danger:hover {
    background: rgba(255, 123, 114, 0.15);
    color: #ff7b72;
  }

  .edit-form {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .edit-input {
    background: #0d1117;
    border: 1px solid #58a6ff;
    border-radius: 4px;
    color: #e6edf3;
    font-size: 13px;
    font-family: inherit;
    padding: 4px 8px;
    outline: none;
    width: 100%;
    -webkit-user-select: text;
    user-select: text;
  }

  .edit-colors {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .edit-actions {
    display: flex;
    gap: 6px;
  }

  .edit-btn {
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 12px;
    font-family: inherit;
    cursor: pointer;
    border: none;
    transition: background 0.15s;
  }

  .edit-btn.cancel {
    background: #3a3a3c;
    color: #e6edf3;
  }

  .edit-btn.cancel:hover {
    background: #48484a;
  }

  .edit-btn.save {
    background: #238636;
    color: #fff;
  }

  .edit-btn.save:hover:not(:disabled) {
    background: #2ea043;
  }

  .edit-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .color-swatch {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    cursor: pointer;
    border: 2px solid transparent;
    transition: all 0.15s;
    flex-shrink: 0;
  }

  .color-swatch:hover {
    transform: scale(1.15);
  }

  .color-swatch.selected {
    border-color: #e6edf3;
    box-shadow: 0 0 0 2px #0d1117, 0 0 0 4px currentColor;
  }

  .color-swatch.small {
    width: 18px;
    height: 18px;
  }

  .color-swatch.small:hover {
    transform: scale(1.2);
  }

  .empty {
    padding: 24px 12px;
    text-align: center;
    color: #636366;
    font-size: 13px;
  }

  .create-form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .create-input {
    background: #0d1117;
    border: 1px solid #3a3a3c;
    border-radius: 6px;
    color: #e6edf3;
    font-size: 13px;
    font-family: inherit;
    padding: 8px 12px;
    outline: none;
    -webkit-user-select: text;
    user-select: text;
    transition: border-color 0.15s;
  }

  .create-input:focus {
    border-color: #58a6ff;
  }

  .create-input::placeholder {
    color: #636366;
  }

  .color-swatches {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .create-btn {
    padding: 8px 16px;
    background: #21262d;
    border: 1px solid #30363d;
    color: #e6edf3;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    font-family: inherit;
    transition: background 0.15s;
    align-self: flex-start;
  }

  .create-btn:hover:not(:disabled) {
    background: #30363d;
  }

  .create-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
