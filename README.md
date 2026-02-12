# Social Media Downloader

A lightweight desktop application for downloading media from Instagram posts.

## Features

- Download images and videos from public Instagram posts
- Support for carousel posts (multiple images/videos)
- Select specific items or download all
- Proper filename formatting with username and shortcode
- Duplicate file handling
- Progress tracking for each download

## Tech Stack

- **Tauri** - Lightweight desktop framework
- **React** - UI library
- **TypeScript** - Type safety
- **Vite** - Fast build tool
- **Rust** - Backend processing

## Project Structure

```
social-media-downloader/
├── src/                        # Frontend React code
│   ├── types/                  # Shared TypeScript types
│   │   └── index.ts
│   ├── core/                   # Core engine modules
│   │   ├── provider.ts         # Provider abstraction
│   │   └── downloadEngine.ts   # Download coordinator
│   ├── providers/              # Platform-specific providers
│   │   └── instagram.ts        # Instagram provider
│   ├── App.tsx                 # Main UI component
│   ├── App.css                 # Styles
│   └── main.tsx                # Entry point
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── main.rs             # Tauri app entry
│   │   └── commands.rs         # Backend commands & logic
│   ├── Cargo.toml              # Rust dependencies
│   └── tauri.conf.json         # Tauri configuration
├── package.json
├── tsconfig.json
└── vite.config.ts
```

## Architecture

The application is built with clean separation of concerns:

1. **UI Layer** (`App.tsx`) - Handles user interaction and display
2. **Provider Layer** (`providers/`) - Platform-specific extraction logic
3. **Core Layer** (`core/`) - Provider abstraction and download engine
4. **Backend Layer** (`src-tauri/`) - Rust commands for HTTP, file I/O, naming

### Provider Abstraction

Each provider implements:
- `id`: Provider identifier
- `matches(url)`: Check if URL is supported
- `resolve(url)`: Extract and normalize media data

### Naming Rules

- Single media: `username_shortcode.ext`
- Carousel: `username_shortcode_01.ext`, `username_shortcode_02.ext`, etc.
- Duplicates: Append `__dup2`, `__dup3`, etc.
- Username is sanitized to lowercase alphanumeric + underscore/hyphen

### Download Strategy

- Downloads to `%USERPROFILE%/Downloads/social-media-downloader/instagram/`
- Streams files to `.tmp` first, then atomically renames
- Concurrency limit of 2 simultaneous downloads
- Per-item progress tracking
- Graceful per-item failure (doesn't stop batch)

## Setup

1. Install dependencies:
```bash
npm install
```

2. Run in development mode:
```bash
npm run tauri:dev
```

3. Build for production:
```bash
npm run tauri:build
```

## Instagram Extraction Method

The Instagram provider:
1. Accepts Instagram post URL (supports `/p/` and `/reel/` formats)
2. Extracts shortcode from URL using regex
3. Fetches post HTML from `instagram.com/p/{shortcode}/`
4. Parses embedded JSON-LD metadata from `<script type="application/ld+json">`
5. Extracts username from `author.identifier.value`
6. Extracts media URLs from `image` and `video` fields
7. Normalizes into standard `ResolveResult` format

## Limitations

- **Public posts only** - No authentication, cannot access private posts
- **Single posts only** - No profile scraping, stories, or bulk downloads
- **Instagram only** - Currently only Instagram is supported (architecture ready for more)
- **HTML extraction dependent** - Relies on Instagram's JSON-LD metadata structure
- **No video quality selection** - Downloads default quality provided by Instagram
- **Rate limiting** - May be subject to Instagram's rate limits on anonymous requests

## Current Support

✅ Public Instagram posts (single image)
✅ Public Instagram posts (single video)  
✅ Public Instagram carousels (multiple images/videos)
✅ Instagram Reels

❌ Private posts
❌ Stories
❌ Profile downloads
❌ Authenticated requests
❌ Other platforms (TikTok, Twitter, etc.)

## Development Notes

- Frontend communicates with Rust backend via Tauri IPC
- Downloads happen in Rust for better performance and file handling
- React app listens for download progress events
- Provider registry makes it easy to add new platforms in the future
