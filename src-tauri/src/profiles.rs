use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub color: String,
    pub is_default: bool,
    pub settings: ProfileSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSettings {
    pub layout: String,
    pub sidebar_width: u32,
}

impl Default for ProfileSettings {
    fn default() -> Self {
        Self {
            layout: "vertical".to_string(),
            sidebar_width: 280,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileStore {
    profiles: Vec<Profile>,
}

pub struct ProfileManager {
    store: ProfileStore,
    storage_path: PathBuf,
}

impl ProfileManager {
    pub fn load(app_data_dir: PathBuf) -> Self {
        fs::create_dir_all(&app_data_dir).ok();
        let storage_path = app_data_dir.join("profiles.json");

        let store = if storage_path.exists() {
            match fs::read_to_string(&storage_path) {
                Ok(content) => serde_json::from_str::<ProfileStore>(&content)
                    .unwrap_or(ProfileStore { profiles: vec![] }),
                Err(_) => ProfileStore { profiles: vec![] },
            }
        } else {
            ProfileStore { profiles: vec![] }
        };

        let mut manager = Self {
            store,
            storage_path,
        };

        if manager.store.profiles.is_empty() {
            manager.create_default();
        }

        manager
    }

    fn create_default(&mut self) {
        let profile = Profile {
            id: Uuid::new_v4().to_string(),
            name: "Default".to_string(),
            color: "#58a6ff".to_string(),
            is_default: true,
            settings: ProfileSettings::default(),
        };
        self.store.profiles.push(profile);
        self.save();
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.store) {
            let _ = fs::write(&self.storage_path, json);
        }
    }

    pub fn list(&self) -> Vec<Profile> {
        self.store.profiles.clone()
    }

    pub fn get(&self, id: &str) -> Result<Profile, String> {
        self.store
            .profiles
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or_else(|| format!("Profile not found: {}", id))
    }

    pub fn get_default(&self) -> Profile {
        self.store
            .profiles
            .iter()
            .find(|p| p.is_default)
            .cloned()
            .unwrap_or_else(|| self.store.profiles[0].clone())
    }

    pub fn create(&mut self, name: &str, color: &str) -> Result<Profile, String> {
        let profile = Profile {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            color: color.to_string(),
            is_default: false,
            settings: ProfileSettings::default(),
        };
        self.store.profiles.push(profile.clone());
        self.save();
        Ok(profile)
    }

    pub fn update(&mut self, id: &str, name: &str, color: &str) -> Result<Profile, String> {
        let profile = self
            .store
            .profiles
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("Profile not found: {}", id))?;
        profile.name = name.to_string();
        profile.color = color.to_string();
        let updated = profile.clone();
        self.save();
        Ok(updated)
    }

    pub fn update_settings(&mut self, id: &str, settings: ProfileSettings) -> Result<(), String> {
        let profile = self
            .store
            .profiles
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("Profile not found: {}", id))?;
        profile.settings = settings;
        self.save();
        Ok(())
    }

    pub fn set_default(&mut self, id: &str) -> Result<(), String> {
        if !self.store.profiles.iter().any(|p| p.id == id) {
            return Err(format!("Profile not found: {}", id));
        }
        for p in &mut self.store.profiles {
            p.is_default = p.id == id;
        }
        self.save();
        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        let profile = self
            .store
            .profiles
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("Profile not found: {}", id))?;

        if profile.is_default {
            return Err("Cannot delete the default profile".to_string());
        }

        self.store.profiles.retain(|p| p.id != id);
        self.save();
        Ok(())
    }
}

pub type ProfileState = Mutex<ProfileManager>;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, ProfileManager) {
        let dir = TempDir::new().unwrap();
        let manager = ProfileManager::load(dir.path().to_path_buf());
        (dir, manager)
    }

    #[test]
    fn test_default_profile_created_on_first_load() {
        let (_dir, manager) = setup();
        let profiles = manager.list();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "Default");
        assert_eq!(profiles[0].color, "#58a6ff");
        assert!(profiles[0].is_default);
    }

    #[test]
    fn test_create_profile_with_name_and_color() {
        let (_dir, mut manager) = setup();
        let profile = manager.create("Work", "#ff6b6b").unwrap();
        assert_eq!(profile.name, "Work");
        assert_eq!(profile.color, "#ff6b6b");
        assert!(!profile.is_default);
        assert_eq!(manager.list().len(), 2);
    }

    #[test]
    fn test_set_default_profile() {
        let (_dir, mut manager) = setup();
        let new_profile = manager.create("Work", "#ff6b6b").unwrap();

        manager.set_default(&new_profile.id).unwrap();

        let profiles = manager.list();
        let old_default = profiles.iter().find(|p| p.name == "Default").unwrap();
        let new_default = profiles.iter().find(|p| p.name == "Work").unwrap();
        assert!(!old_default.is_default);
        assert!(new_default.is_default);
    }

    #[test]
    fn test_delete_non_default_profile() {
        let (_dir, mut manager) = setup();
        let profile = manager.create("Temp", "#aabbcc").unwrap();
        assert_eq!(manager.list().len(), 2);

        manager.delete(&profile.id).unwrap();
        assert_eq!(manager.list().len(), 1);
    }

    #[test]
    fn test_cannot_delete_default_profile() {
        let (_dir, mut manager) = setup();
        let default_id = manager.get_default().id;
        let result = manager.delete(&default_id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Cannot delete the default profile"
        );
    }

    #[test]
    fn test_update_profile_settings() {
        let (_dir, mut manager) = setup();
        let profile = manager.create("Dev", "#00ff00").unwrap();

        let new_settings = ProfileSettings {
            layout: "horizontal".to_string(),
            sidebar_width: 350,
        };
        manager.update_settings(&profile.id, new_settings).unwrap();

        let updated = manager.get(&profile.id).unwrap();
        assert_eq!(updated.settings.layout, "horizontal");
        assert_eq!(updated.settings.sidebar_width, 350);
    }

    #[test]
    fn test_persistence_across_loads() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();

        let created_id;
        {
            let mut manager = ProfileManager::load(path.clone());
            let profile = manager.create("Persistent", "#112233").unwrap();
            created_id = profile.id;
        }

        let manager = ProfileManager::load(path);
        let profiles = manager.list();
        assert_eq!(profiles.len(), 2);
        let found = profiles.iter().find(|p| p.id == created_id).unwrap();
        assert_eq!(found.name, "Persistent");
        assert_eq!(found.color, "#112233");
    }

    #[test]
    fn test_update_profile_name_and_color() {
        let (_dir, mut manager) = setup();
        let profile = manager.create("Old Name", "#000000").unwrap();

        let updated = manager.update(&profile.id, "New Name", "#ffffff").unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.color, "#ffffff");

        let fetched = manager.get(&profile.id).unwrap();
        assert_eq!(fetched.name, "New Name");
        assert_eq!(fetched.color, "#ffffff");
    }
}
