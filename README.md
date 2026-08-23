# Telegram Drive (Optimized for VPNs)

**Telegram Drive** is an open-source application that turns your Telegram account into an unlimited, secure cloud storage drive. Built with a **React** SPA frontend and a **Rust** backend (using **Axum** for REST API services, **Actix Web** for media streaming, and **Grammers** for Telegram API integration).

This repository is **heavily optimized for users with low bandwidth accessing the Telegram API through a VPN**. It includes custom network polling, automatic multi-DC fallback, exponential backoff retries for high-latency connections, and deep API optimizations to minimize round trips.

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20MacOS%20%7C%20Linux%20%7C%20Docker-blue)

![Auth Screen](TelegramDriveForVPNs.png)

## What is Telegram Drive?

Telegram Drive leverages the Telegram API to allow you to upload, organize, and manage files directly on Telegram's servers. It treats your "Saved Messages" and created Channels as folders, giving you a familiar file explorer interface for your Telegram cloud.

### Key Features

* **Unlimited Cloud Storage**: Utilizing Telegram's generous cloud infrastructure.
* **High Performance Grid**: Virtual scrolling handles folders with thousands of files instantly.
* **Media Streaming**: Stream video and audio files directly via a dedicated Actix streaming server without waiting for full downloads.
* **PDF Viewer**: Native in-app PDF preview with zoom, page navigation, and thumbnail sidebar.
* **Progressive File Loading**: Cursor-based pagination loads files progressively for responsive browsing on high-latency networks.
* **Drag & Drop**: Intuitive drag-and-drop upload and file management.
* **Thumbnail Previews & Caching**: Inline thumbnails cached locally for instant reloading.
* **Folder Management**: Create private Telegram channels as virtual folders to organize content.
* **Privacy Focused**: Telegram session data and API keys stay local or on your server. No third-party servers.

### VPN & Low-Bandwidth Optimizations

This version includes specific backend enhancements to handle the high latency, packet loss, and connection instability common when routing Telegram traffic through VPNs or restricted networks:

* **Multi-DC Network Checks**: Attempts connections across 5 different Telegram Data Centers (DC1-DC5) to find the most stable route rather than relying on a single hardcoded IP.
* **Latency-Tolerant Timeouts**: TCP connection timeouts increased from 2s to 8s.
* **Exponential Backoff & Retries**: Uploads, downloads, and RPCs are wrapped in custom retry handlers that automatically recover from broken pipes and connection drops.
* **Flood Wait Handling**: Automatically detects and handles Telegram API `FLOOD_WAIT` rate limits before transparently resuming operations.
* **O(1) Peer Resolution**: Implements an in-memory `HashMap` cache for Telegram peers, eliminating the need to iterate through hundreds of dialogs (saving 30+ seconds of API round trips on file operations).
* **Direct Message Fetching**: Replaced O(n) message iteration with targeted `get_messages_by_id` calls for instant file downloads and thumbnail generation.
* **Adaptive Polling**: Frontend network polling scales intelligently (30s to 45s) to reduce unnecessary VPN traffic while keeping UI state current.
* **Resilient Authentication**: Initial connections and session verification include retry loops to prevent accidental logouts on temporary network drops.

## Screenshots

| Dashboard | File Preview |
|-----------|--------------|
| ![Dashboard](https://raw.githubusercontent.com/caamer20/Telegram-Drive-ForVPNs/refs/heads/main/screenshots/DashboardWithFiles.png) | ![Preview](https://raw.githubusercontent.com/caamer20/Telegram-Drive-ForVPNs/refs/heads/main/screenshots/ImagePreview.png) |

| Grid View | Authentication |
|-----------|----------------|
| ![Dark Mode](https://raw.githubusercontent.com/caamer20/Telegram-Drive-ForVPNs/refs/heads/main/screenshots/DarkModeGrid.png) | ![Login](https://raw.githubusercontent.com/caamer20/Telegram-Drive-ForVPNs/refs/heads/main/screenshots/LoginScreen.png?raw=true) |

| Audio Playback | Video Playback |
|----------------|----------------|
| ![Audio Playback](https://raw.githubusercontent.com/caamer20/Telegram-Drive-ForVPNs/refs/heads/main/screenshots/AudioPlayback.png?raw=true) | ![Video Playback](https://raw.githubusercontent.com/caamer20/Telegram-Drive-ForVPNs/refs/heads/main/screenshots/VideoPlayback.png?raw=true) |

| Auth Code Screen | Upload Example |
|------------------|-------------|
| ![Auth Code Screen](TelegramDriveForVPNs.png) | ![Upload Example](https://raw.githubusercontent.com/caamer20/Telegram-Drive-ForVPNs/refs/heads/main/screenshots/UploadExample.png?raw=true) |

| Folder Creation | Folder List View |
|-----------------|------------------|
| ![Folder Creation](https://raw.githubusercontent.com/caamer20/Telegram-Drive-ForVPNs/refs/heads/main/screenshots/FolderCreation.png?raw=true) | ![Folder List View](https://raw.githubusercontent.com/caamer20/Telegram-Drive-ForVPNs/refs/heads/main/screenshots/FolderListView.png?raw=true) |

## Tech Stack & Architecture

* **Frontend**: React 19, TypeScript, Vite, Tailwind CSS, Framer Motion, TanStack Query, TanStack Virtual, PDF.js
* **Backend**: Rust
  * **Axum**: REST API and web server (default port `8550`)
  * **Actix Web**: High-performance media streaming server (port `14201`)
  * **Grammers**: Pure-Rust Telegram client implementation
  * **SQLite**: Local session persistence (`telegram.session`)
* **Package Manager**: `pnpm`

### Network Ports & Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      Browser / UI                       │
│              (Vite Dev Server: Port 1420)               │
└────────────┬──────────────────────────────┬─────────────┘
             │                              │
   /api REST requests               Media Streaming
             │                              │
             ▼                              ▼
┌──────────────────────────┐   ┌──────────────────────────┐
│     Axum REST Server     │   │  Actix Streaming Server  │
│       (Port 8550)        │   │       (Port 14201)       │
└────────────┬─────────────┘   └────────────┬─────────────┘
             │                              │
             └──────────────┬───────────────┘
                            │
                            ▼
               ┌─────────────────────────┐
               │  Grammers MTProto Client│
               └────────────┬────────────┘
                            │
                            ▼
               ┌─────────────────────────┐
               │   Telegram Data Center  │
               └─────────────────────────┘
```

## Getting Started

### Prerequisites

* **Node.js (v20+)**: Required for building and running the frontend interface.
* **pnpm (v10+)**: Package manager used across the frontend repository (`corepack enable` or `npm i -g pnpm`).
* **Rust (latest stable)**: Required to compile the Rust backend. Install via [rustup](https://rustup.rs/):
  * **macOS/Linux:** `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  * **Windows:** Download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs/)
* **Docker (Optional)**: For running containerized server deployments.
* **Telegram API Credentials**: You need your own API ID and API Hash to communicate with Telegram's servers.
  1. Log into [my.telegram.org](https://my.telegram.org).
  2. Go to "API development tools" and create a new application to retrieve your `api_id` and `api_hash`.

### Installation & Development

1. **Clone the repository**
   ```bash
   git clone https://github.com/caamer20/Telegram-Drive-ForVPNs.git
   cd Telegram-Drive-ForVPNs
   ```

2. **Install Frontend Dependencies**
   ```bash
   cd app
   pnpm install
   ```

3. **Run the Application**

   * **Option A: Run Frontend & Backend separately for development**

     Terminal 1 (Backend Axum & Actix server on port 8550 and 14201):
     ```bash
     cd app/src-tauri
     cargo run
     ```

     Terminal 2 (Frontend Vite dev server on port 1420):
     ```bash
     cd app
     pnpm dev
     ```

     Open `http://localhost:1420` in your browser.

   * **Option B: Run with Docker**
     ```bash
     docker build -t telegram-drive .
     docker run -p 8550:8550 -v telegram_data:/app/data telegram-drive
     ```

     Open `http://localhost:8550` in your browser.

4. **Production Build**

   * **Build Frontend Assets:**
     ```bash
     cd app
     pnpm build
     ```
     This generates optimized static files in `app/dist`.

   * **Build Release Backend Server Binary:**
     ```bash
     cd app/src-tauri
     cargo build --release
     ```
     The binary will be generated at `app/src-tauri/target/release/app`.

## Configuration & Environment Variables

The backend server reads configuration from environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | HTTP port for the Axum REST API and web server | `8550` |
| `STATIC_DIR` | Path to static frontend build directory (`app/dist`) | None (API-only mode if omitted) |
| `DATA_DIR` | Base directory for storing `telegram.session`, `session_config.json`, and thumbnail cache | System App Data directory or `/app/data` |
| `RUST_LOG` | Logging filter level (e.g. `info`, `debug`, `app=debug`) | `info` |

## Open Source & License

This project is **Free and Open Source Software**. You are free to use, modify, and distribute it under the **MIT License**.

---
*Disclaimer: This application is not affiliated with Telegram FZ-LLC. Use responsibly and in accordance with Telegram's Terms of Service.*

<div align="center">
  <!-- PayPal -->
  <div style="margin: 15px 0;">
    <a href="https://www.paypal.me/Caamer20">
      <img src="https://raw.githubusercontent.com/stefan-niedermann/paypal-donate-button/master/paypal-donate-button.png" alt="Donate with PayPal" width="200">
    </a>
    <div style="font-size: 14px; margin-top: 8px;">paypal.me/Caamer20</div>
  </div>

  <!-- Litecoin -->
  <div style="margin: 15px 0;">
    <a href="litecoin:ltc1q6wkr5ac4u0pxx4hx7xgwn0gsaku25ws0df73rp">
      <img src="https://img.shields.io/badge/Donate-LTC-345D9D?style=for-the-badge&logo=litecoin&logoColor=white" alt="Donate LTC">
    </a>
    <div style="font-family: monospace; font-size: 13px; margin-top: 8px; word-break: break-all;">
      ltc1q6wkr5ac4u0pxx4hx7xgwn0gsaku25ws0df73rp
    </div>
  </div>

  <!-- Bitcoin -->
  <div style="margin: 15px 0;">
    <a href="bitcoin:bc1q5pt7m2fk6w0dzsnf6vvd5k6nw5k44785286ujy">
      <img src="https://img.shields.io/badge/Donate-BTC-F7931A?style=for-the-badge&logo=bitcoin&logoColor=white" alt="Donate BTC">
    </a>
    <div style="font-family: monospace; font-size: 13px; margin-top: 8px; word-break: break-all;">
      bc1q5pt7m2fk6w0dzsnf6vvd5k6nw5k44785286ujy
    </div>
  </div>
</div>
