// Instagram provider implementation
import { invoke } from '@tauri-apps/api/tauri';
import { Provider } from '../core/provider';
import { ResolveResult } from '../types';

export class InstagramProvider implements Provider {
  id = 'instagram' as const;

  matches(url: string): boolean {
    const pattern = /instagram\.com\/(p|reel)\/[\w-]+/;
    return pattern.test(url);
  }

  async resolve(url: string): Promise<ResolveResult> {
    try {
      const result = await invoke<ResolveResult>('resolve_post', { url });
      return result;
    } catch (error) {
      throw new Error(error as string);
    }
  }
}
