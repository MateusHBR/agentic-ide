import { invoke } from "@tauri-apps/api/core";

export interface ProfileSettings {
  layout: "vertical" | "horizontal";
  sidebar_width: number;
}

export interface Profile {
  id: string;
  name: string;
  color: string;
  is_default: boolean;
  settings: ProfileSettings;
}

// --- Tauri invoke wrappers ---

export async function listProfiles(): Promise<Profile[]> {
  return invoke("list_profiles");
}

export async function getProfile(id: string): Promise<Profile> {
  return invoke("get_profile", { id });
}

export async function getDefaultProfile(): Promise<Profile> {
  return invoke("get_default_profile");
}

export async function createProfile(name: string, color: string): Promise<Profile> {
  return invoke("create_profile", { name, color });
}

export async function updateProfile(id: string, name: string, color: string): Promise<Profile> {
  return invoke("update_profile", { id, name, color });
}

export async function updateProfileSettings(id: string, settings: ProfileSettings): Promise<void> {
  return invoke("update_profile_settings", { id, settings });
}

export async function deleteProfile(id: string): Promise<void> {
  return invoke("delete_profile", { id });
}

export async function setDefaultProfile(id: string): Promise<void> {
  return invoke("set_default_profile", { id });
}

// --- Reactive profile state ---

class ProfileState {
  profiles = $state<Profile[]>([]);
  activeProfileId = $state<string>("");

  get activeProfile(): Profile | undefined {
    return this.profiles.find(p => p.id === this.activeProfileId);
  }

  async loadProfiles() {
    try {
      this.profiles = await listProfiles();
    } catch (e) {
      console.error("Failed to load profiles:", e);
    }
  }

  async initialize(profileId?: string) {
    await this.loadProfiles();
    if (profileId) {
      this.activeProfileId = profileId;
    } else {
      const defaultProfile = this.profiles.find(p => p.is_default);
      this.activeProfileId = defaultProfile?.id ?? this.profiles[0]?.id ?? "";
    }
  }

  async createNewProfile(name: string, color: string): Promise<Profile | null> {
    try {
      const profile = await createProfile(name, color);
      await this.loadProfiles();
      return profile;
    } catch (e) {
      console.error("Failed to create profile:", e);
      return null;
    }
  }

  async deleteExistingProfile(id: string): Promise<boolean> {
    try {
      await deleteProfile(id);
      this.profiles = this.profiles.filter(p => p.id !== id);

      // If the deleted profile was active, switch to the default profile
      if (this.activeProfileId === id) {
        const defaultProfile = this.profiles.find(p => p.is_default) ?? this.profiles[0];
        if (defaultProfile) {
          this.activeProfileId = defaultProfile.id;
        }
      }

      // Clean up orphaned localStorage key for the deleted profile
      localStorage.removeItem(`agentic-ide-projects-${id}`);

      return true;
    } catch (e) {
      console.error("Failed to delete profile:", e);
      return false;
    }
  }

  async setDefault(id: string): Promise<boolean> {
    try {
      await setDefaultProfile(id);
      for (const p of this.profiles) {
        p.is_default = p.id === id;
      }
      return true;
    } catch (e) {
      console.error("Failed to set default profile:", e);
      return false;
    }
  }

  async saveCurrentSettings(settings: ProfileSettings) {
    if (!this.activeProfileId) return;
    try {
      await updateProfileSettings(this.activeProfileId, settings);
      const profile = this.profiles.find(p => p.id === this.activeProfileId);
      if (profile) {
        profile.settings = settings;
      }
    } catch (e) {
      console.error("Failed to save profile settings:", e);
    }
  }
}

export const profileState = new ProfileState();
