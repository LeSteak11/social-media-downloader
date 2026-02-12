// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

mod commands;

// Shared types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    #[serde(rename = "type")]
    pub media_type: String,
    pub preview_url: String,
    pub download_url: String,
    pub extension: String,
    pub index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveResult {
    pub username: String,
    pub shortcode: String,
    pub media_items: Vec<MediaItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    pub username: String,
    pub shortcode: String,
    pub media_items: Vec<MediaItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub item_id: String,
    pub status: String,
    pub progress: f32,
    pub filename: Option<String>,
    pub error: Option<String>,
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::resolve_post,
            commands::download_media,
            commands::get_downloads_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
