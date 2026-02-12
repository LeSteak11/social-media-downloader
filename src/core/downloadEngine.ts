// Download engine coordinator
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { DownloadRequest, DownloadProgress, MediaItem } from '../types';

export class DownloadEngine {
  private progressCallbacks: Map<string, (progress: DownloadProgress) => void> = new Map();
  private listenerUnsubscribe?: () => void;

  async initialize(): Promise<void> {
    this.listenerUnsubscribe = await listen<DownloadProgress>(
      'download-progress',
      (event) => {
        const progress = event.payload;
        const callback = this.progressCallbacks.get(progress.itemId);
        if (callback) {
          callback(progress);
        }
      }
    );
  }

  async getDownloadDirectory(): Promise<string> {
    return await invoke<string>('get_downloads_dir');
  }

  async download(
    username: string,
    shortcode: string,
    mediaItems: MediaItem[],
    onProgress: (progress: DownloadProgress) => void
  ): Promise<void> {
    // Register progress callbacks
    for (const item of mediaItems) {
      this.progressCallbacks.set(item.id, onProgress);
    }

    const downloadDir = await this.getDownloadDirectory();

    const request: DownloadRequest = {
      username,
      shortcode,
      mediaItems,
    };

    try {
      await invoke('download_media', { 
        request, 
        downloadDir 
      });
    } finally {
      // Clean up callbacks
      for (const item of mediaItems) {
        this.progressCallbacks.delete(item.id);
      }
    }
  }

  destroy(): void {
    if (this.listenerUnsubscribe) {
      this.listenerUnsubscribe();
    }
    this.progressCallbacks.clear();
  }
}

export const downloadEngine = new DownloadEngine();
