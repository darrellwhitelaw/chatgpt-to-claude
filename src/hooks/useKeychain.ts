import { invoke } from '@tauri-apps/api/core';

export function useKeychain() {
  const getApiKey = (): Promise<string> =>
    invoke('get_api_key');

  const setApiKey = (key: string): Promise<void> =>
    invoke('set_api_key', { key });

  const deleteApiKey = (): Promise<void> =>
    invoke('delete_api_key');

  return { getApiKey, setApiKey, deleteApiKey };
}
