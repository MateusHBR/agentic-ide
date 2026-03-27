import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import type { Profile, ProfileSettings } from './profiles.svelte';

const mockedInvoke = vi.mocked(invoke);

const mockDefaultProfile: Profile = {
  id: 'default-uuid',
  name: 'Default',
  color: '#58a6ff',
  is_default: true,
  settings: { layout: 'vertical', sidebar_width: 280 },
};

const mockWorkProfile: Profile = {
  id: 'work-uuid',
  name: 'Work',
  color: '#ff7b72',
  is_default: false,
  settings: { layout: 'horizontal', sidebar_width: 300 },
};

describe('Profile types', () => {
  it('should define Profile interface correctly', () => {
    const profile: Profile = mockDefaultProfile;
    expect(profile.id).toBe('default-uuid');
    expect(profile.is_default).toBe(true);
    expect(profile.settings.layout).toBe('vertical');
  });

  it('should define ProfileSettings interface correctly', () => {
    const settings: ProfileSettings = { layout: 'horizontal', sidebar_width: 300 };
    expect(settings.layout).toBe('horizontal');
    expect(settings.sidebar_width).toBe(300);
  });
});

describe('Profile API functions', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('listProfiles calls invoke with correct command', async () => {
    mockedInvoke.mockResolvedValue([mockDefaultProfile]);
    const { listProfiles } = await import('./profiles.svelte');
    const result = await listProfiles();
    expect(mockedInvoke).toHaveBeenCalledWith('list_profiles');
    expect(result).toEqual([mockDefaultProfile]);
  });

  it('getProfile calls invoke with id', async () => {
    mockedInvoke.mockResolvedValue(mockDefaultProfile);
    const { getProfile } = await import('./profiles.svelte');
    const result = await getProfile('default-uuid');
    expect(mockedInvoke).toHaveBeenCalledWith('get_profile', { id: 'default-uuid' });
    expect(result.name).toBe('Default');
  });

  it('getDefaultProfile calls invoke', async () => {
    mockedInvoke.mockResolvedValue(mockDefaultProfile);
    const { getDefaultProfile } = await import('./profiles.svelte');
    const result = await getDefaultProfile();
    expect(mockedInvoke).toHaveBeenCalledWith('get_default_profile');
    expect(result.is_default).toBe(true);
  });

  it('createProfile calls invoke with name and color', async () => {
    mockedInvoke.mockResolvedValue(mockWorkProfile);
    const { createProfile } = await import('./profiles.svelte');
    const result = await createProfile('Work', '#ff7b72');
    expect(mockedInvoke).toHaveBeenCalledWith('create_profile', { name: 'Work', color: '#ff7b72' });
    expect(result.name).toBe('Work');
    expect(result.is_default).toBe(false);
  });

  it('updateProfile calls invoke with id, name, color', async () => {
    const updated = { ...mockWorkProfile, name: 'Updated' };
    mockedInvoke.mockResolvedValue(updated);
    const { updateProfile } = await import('./profiles.svelte');
    const result = await updateProfile('work-uuid', 'Updated', '#ff7b72');
    expect(mockedInvoke).toHaveBeenCalledWith('update_profile', { id: 'work-uuid', name: 'Updated', color: '#ff7b72' });
    expect(result.name).toBe('Updated');
  });

  it('updateProfileSettings calls invoke with id and settings', async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const { updateProfileSettings } = await import('./profiles.svelte');
    const settings: ProfileSettings = { layout: 'horizontal', sidebar_width: 350 };
    await updateProfileSettings('work-uuid', settings);
    expect(mockedInvoke).toHaveBeenCalledWith('update_profile_settings', { id: 'work-uuid', settings });
  });

  it('deleteProfile calls invoke with id', async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const { deleteProfile } = await import('./profiles.svelte');
    await deleteProfile('work-uuid');
    expect(mockedInvoke).toHaveBeenCalledWith('delete_profile', { id: 'work-uuid' });
  });

  it('setDefaultProfile calls invoke with id', async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const { setDefaultProfile } = await import('./profiles.svelte');
    await setDefaultProfile('work-uuid');
    expect(mockedInvoke).toHaveBeenCalledWith('set_default_profile', { id: 'work-uuid' });
  });
});
