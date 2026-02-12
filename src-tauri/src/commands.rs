use crate::{DownloadProgress, DownloadRequest, MediaItem, ResolveResult};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Window;
use tokio::io::AsyncWriteExt;

// Provider abstraction
pub trait Provider {
    fn id(&self) -> &str;
    fn matches(&self, url: &str) -> bool;
    fn resolve(&self, url: &str) -> impl std::future::Future<Output = Result<ResolveResult, String>> + Send;
}

// Instagram Provider
pub struct InstagramProvider {
    client: Client,
}

impl InstagramProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .build()
                .unwrap(),
        }
    }

    fn extract_shortcode(&self, url: &str) -> Option<String> {
        let re = Regex::new(r"instagram\.com/(?:p|reel)/([A-Za-z0-9_-]+)").unwrap();
        re.captures(url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }
}

impl Provider for InstagramProvider {
    fn id(&self) -> &str {
        "instagram"
    }

    fn matches(&self, url: &str) -> bool {
        url.contains("instagram.com") && self.extract_shortcode(url).is_some()
    }

    async fn resolve(&self, url: &str) -> Result<ResolveResult, String> {
        let shortcode = self
            .extract_shortcode(url)
            .ok_or_else(|| "Invalid Instagram URL".to_string())?;

        let post_url = format!("https://www.instagram.com/p/{}/", shortcode);
        
        let html = self
            .client
            .get(&post_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch post: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let document = Html::parse_document(&html);
        
        // Extract JSON from script tag
        let script_selector = Selector::parse("script[type='application/ld+json']").unwrap();
        let json_data: Value = document
            .select(&script_selector)
            .filter_map(|el| {
                let text = el.inner_html();
                serde_json::from_str(&text).ok()
            })
            .find(|v: &Value| v.get("@type").and_then(|t| t.as_str()) == Some("ImageObject"))
            .ok_or_else(|| "Could not find post data in page".to_string())?;

        // Extract username from author field
        let username = json_data
            .get("author")
            .and_then(|a| a.get("identifier"))
            .and_then(|i| i.get("value"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                json_data
                    .get("author")
                    .and_then(|a| a.as_str())
            })
            .ok_or_else(|| "Could not extract username".to_string())?
            .to_string();

        let sanitized_username = sanitize_username(&username);

        // Try to extract multiple images from articleBody or other fields
        let mut media_items = Vec::new();

        // Check if it's a carousel
        if let Some(image_array) = json_data.get("image").and_then(|v| v.as_array()) {
            // Multiple images (carousel)
            for (idx, img_url) in image_array.iter().enumerate() {
                if let Some(url_str) = img_url.as_str() {
                    media_items.push(MediaItem {
                        id: format!("{}_{}", shortcode, idx + 1),
                        media_type: "image".to_string(),
                        preview_url: url_str.to_string(),
                        download_url: url_str.to_string(),
                        extension: "jpg".to_string(),
                        index: Some(idx + 1),
                    });
                }
            }
        } else if let Some(img_url) = json_data.get("image").and_then(|v| v.as_str()) {
            // Single image
            media_items.push(MediaItem {
                id: shortcode.clone(),
                media_type: "image".to_string(),
                preview_url: img_url.to_string(),
                download_url: img_url.to_string(),
                extension: "jpg".to_string(),
                index: None,
            });
        }

        // Check for video
        if let Some(video_url) = json_data.get("video").and_then(|v| v.as_str()) {
            media_items.push(MediaItem {
                id: shortcode.clone(),
                media_type: "video".to_string(),
                preview_url: video_url.to_string(),
                download_url: video_url.to_string(),
                extension: "mp4".to_string(),
                index: None,
            });
        } else if let Some(video_array) = json_data.get("video").and_then(|v| v.as_array()) {
            for (idx, vid_url) in video_array.iter().enumerate() {
                if let Some(url_str) = vid_url.as_str() {
                    media_items.push(MediaItem {
                        id: format!("{}_{}", shortcode, idx + 1),
                        media_type: "video".to_string(),
                        preview_url: url_str.to_string(),
                        download_url: url_str.to_string(),
                        extension: "mp4".to_string(),
                        index: Some(idx + 1),
                    });
                }
            }
        }

        if media_items.is_empty() {
            return Err("No media items found in post".to_string());
        }

        Ok(ResolveResult {
            username: sanitized_username,
            shortcode,
            media_items,
        })
    }
}

// Naming engine
fn sanitize_username(username: &str) -> String {
    username
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

fn generate_filename(username: &str, shortcode: &str, extension: &str, index: Option<usize>) -> String {
    let sanitized = sanitize_username(username);
    if let Some(idx) = index {
        format!("{}_{}_{:02}.{}", sanitized, shortcode, idx, extension)
    } else {
        format!("{}_{}.{}", sanitized, shortcode, extension)
    }
}

fn ensure_unique_filename(dir: &Path, filename: &str) -> PathBuf {
    let mut path = dir.join(filename);
    if !path.exists() {
        return path;
    }

    let stem = path.file_stem().unwrap().to_str().unwrap();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let mut counter = 2;
    loop {
        let new_name = if ext.is_empty() {
            format!("{}__dup{}", stem, counter)
        } else {
            format!("{}__dup{}.{}", stem, counter, ext)
        };
        path = dir.join(new_name);
        if !path.exists() {
            return path;
        }
        counter += 1;
    }
}

// Download engine
async fn download_single_file(
    client: &Client,
    url: &str,
    dest_path: &Path,
) -> Result<(), String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let temp_path = dest_path.with_extension("tmp");

    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    file.write_all(&bytes)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    file.sync_all()
        .await
        .map_err(|e| format!("Failed to sync file: {}", e))?;

    drop(file);

    tokio::fs::rename(&temp_path, dest_path)
        .await
        .map_err(|e| format!("Failed to move temp file: {}", e))?;

    Ok(())
}

// Tauri commands
#[tauri::command]
pub async fn resolve_post(url: String) -> Result<ResolveResult, String> {
    let provider = InstagramProvider::new();
    
    if !provider.matches(&url) {
        return Err("URL is not a valid Instagram post".to_string());
    }

    provider.resolve(&url).await
}

#[tauri::command]
pub async fn download_media(
    window: Window,
    request: DownloadRequest,
    download_dir: String,
) -> Result<(), String> {
    let dir_path = PathBuf::from(&download_dir).join("social-media-downloader/instagram");
    
    fs::create_dir_all(&dir_path)
        .map_err(|e| format!("Failed to create download directory: {}", e))?;

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .unwrap();

    // Download with concurrency limit of 2
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(2));
    let mut handles = vec![];

    for item in request.media_items {
        let sem = semaphore.clone();
        let client = client.clone();
        let username = request.username.clone();
        let shortcode = request.shortcode.clone();
        let dir_path = dir_path.clone();
        let window = window.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            let filename = generate_filename(&username, &shortcode, &item.extension, item.index);
            let file_path = ensure_unique_filename(&dir_path, &filename);

            // Send starting progress
            let _ = window.emit(
                "download-progress",
                DownloadProgress {
                    item_id: item.id.clone(),
                    status: "downloading".to_string(),
                    progress: 0.0,
                    filename: Some(file_path.file_name().unwrap().to_str().unwrap().to_string()),
                    error: None,
                },
            );

            match download_single_file(&client, &item.download_url, &file_path).await {
                Ok(_) => {
                    let _ = window.emit(
                        "download-progress",
                        DownloadProgress {
                            item_id: item.id.clone(),
                            status: "completed".to_string(),
                            progress: 100.0,
                            filename: Some(file_path.file_name().unwrap().to_str().unwrap().to_string()),
                            error: None,
                        },
                    );
                }
                Err(e) => {
                    let _ = window.emit(
                        "download-progress",
                        DownloadProgress {
                            item_id: item.id.clone(),
                            status: "failed".to_string(),
                            progress: 0.0,
                            filename: None,
                            error: Some(e),
                        },
                    );
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_downloads_dir() -> Result<String, String> {
    tauri::api::path::download_dir()
        .map(|p| p.to_str().unwrap().to_string())
        .ok_or_else(|| "Could not find downloads directory".to_string())
}
