// Shared types across the application

export interface MediaItem {
  id: string;
  type: 'image' | 'video';
  previewUrl: string;
  downloadUrl: string;
  extension: string;
  index?: number;
}

export interface ResolveResult {
  username: string;
  shortcode: string;
  mediaItems: MediaItem[];
}

export interface DownloadRequest {
  username: string;
  shortcode: string;
  mediaItems: MediaItem[];
}

export interface DownloadProgress {
  itemId: string;
  status: 'downloading' | 'completed' | 'failed';
  progress: number;
  filename?: string;
  error?: string;
}

export type ProviderType = 'instagram';

export interface ProviderError {
  message: string;
  provider?: ProviderType;
}
