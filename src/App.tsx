import { useState, useEffect } from 'react';
import { providerRegistry } from './core/provider';
import { InstagramProvider } from './providers/instagram';
import { downloadEngine } from './core/downloadEngine';
import { ResolveResult, DownloadProgress } from './types';
import './App.css';

// Register providers
providerRegistry.register(new InstagramProvider());

interface ItemProgress {
  status: 'pending' | 'downloading' | 'completed' | 'failed';
  progress: number;
  filename?: string;
  error?: string;
}

function App() {
  const [url, setUrl] = useState('');
  const [resolveResult, setResolveResult] = useState<ResolveResult | null>(null);
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set());
  const [isResolving, setIsResolving] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<Map<string, ItemProgress>>(new Map());

  useEffect(() => {
    downloadEngine.initialize();
    return () => downloadEngine.destroy();
  }, []);

  const handleResolve = async () => {
    setError(null);
    setResolveResult(null);
    setIsResolving(true);

    try {
      const result = await providerRegistry.resolve(url);
      setResolveResult(result);
      // Auto-select all items
      setSelectedItems(new Set(result.mediaItems.map(item => item.id)));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsResolving(false);
    }
  };

  const toggleSelection = (itemId: string) => {
    const newSelected = new Set(selectedItems);
    if (newSelected.has(itemId)) {
      newSelected.delete(itemId);
    } else {
      newSelected.add(itemId);
    }
    setSelectedItems(newSelected);
  };

  const toggleSelectAll = () => {
    if (!resolveResult) return;
    
    if (selectedItems.size === resolveResult.mediaItems.length) {
      setSelectedItems(new Set());
    } else {
      setSelectedItems(new Set(resolveResult.mediaItems.map(item => item.id)));
    }
  };

  const handleDownload = async () => {
    if (!resolveResult) return;

    const itemsToDownload = resolveResult.mediaItems.filter(item => 
      selectedItems.has(item.id)
    );

    if (itemsToDownload.length === 0) {
      setError('Please select at least one item');
      return;
    }

    setIsDownloading(true);
    setError(null);

    // Initialize progress for all items
    const initialProgress = new Map<string, ItemProgress>();
    itemsToDownload.forEach(item => {
      initialProgress.set(item.id, {
        status: 'pending',
        progress: 0,
      });
    });
    setDownloadProgress(initialProgress);

    const handleProgress = (progress: DownloadProgress) => {
      setDownloadProgress(prev => {
        const newMap = new Map(prev);
        newMap.set(progress.itemId, {
          status: progress.status,
          progress: progress.progress,
          filename: progress.filename,
          error: progress.error,
        });
        return newMap;
      });
    };

    try {
      await downloadEngine.download(
        resolveResult.username,
        resolveResult.shortcode,
        itemsToDownload,
        handleProgress
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsDownloading(false);
    }
  };

  return (
    <div className="app">
      <header>
        <h1>Social Media Downloader</h1>
      </header>

      <main>
        <section className="input-section">
          <input
            type="text"
            placeholder="Paste Instagram post URL..."
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            disabled={isResolving || isDownloading}
          />
          <button onClick={handleResolve} disabled={!url || isResolving || isDownloading}>
            {isResolving ? 'Resolving...' : 'Resolve'}
          </button>
        </section>

        {error && (
          <div className="error-message">
            {error}
          </div>
        )}

        {resolveResult && (
          <section className="result-section">
            <div className="result-header">
              <h2>@{resolveResult.username} - {resolveResult.mediaItems.length} item(s)</h2>
              <div className="actions">
                <label className="select-all">
                  <input
                    type="checkbox"
                    checked={selectedItems.size === resolveResult.mediaItems.length}
                    onChange={toggleSelectAll}
                    disabled={isDownloading}
                  />
                  Select All
                </label>
                <button 
                  onClick={handleDownload} 
                  disabled={selectedItems.size === 0 || isDownloading}
                  className="download-btn"
                >
                  {isDownloading ? 'Downloading...' : `Download (${selectedItems.size})`}
                </button>
              </div>
            </div>

            <div className="media-grid">
              {resolveResult.mediaItems.map((item) => {
                const progress = downloadProgress.get(item.id);
                return (
                  <div key={item.id} className="media-item">
                    <div className="media-preview">
                      {item.type === 'image' ? (
                        <img src={item.previewUrl} alt="Media preview" />
                      ) : (
                        <video src={item.previewUrl} />
                      )}
                      <div className="media-overlay">
                        <input
                          type="checkbox"
                          checked={selectedItems.has(item.id)}
                          onChange={() => toggleSelection(item.id)}
                          disabled={isDownloading}
                        />
                        <span className="media-type">{item.type}</span>
                      </div>
                    </div>
                    {progress && (
                      <div className={`progress-bar ${progress.status}`}>
                        {progress.status === 'downloading' && (
                          <div className="progress-fill" style={{ width: `${progress.progress}%` }} />
                        )}
                        <span className="progress-text">
                          {progress.status === 'completed' && `✓ ${progress.filename}`}
                          {progress.status === 'downloading' && 'Downloading...'}
                          {progress.status === 'failed' && `✗ ${progress.error}`}
                          {progress.status === 'pending' && 'Pending...'}
                        </span>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </section>
        )}
      </main>
    </div>
  );
}

export default App;
