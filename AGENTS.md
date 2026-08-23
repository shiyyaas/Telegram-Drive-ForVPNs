# AGENTS.md — Developer & AI Agent Guide

This repository contains **Telegram Drive (Optimized for VPNs)**, a desktop/web cloud storage interface for Telegram built with a **React** frontend and a **Rust** backend server.

This document serves as the canonical reference for AI agents and human developers working in this codebase.

---

## Workspace Structure & Component Map

```
.
├── Dockerfile                  # Multi-stage Docker build for containerized server
├── README.md                   # Root user documentation and quickstart guide
├── AGENTS.md                   # Agent developer guide (this file)
├── CHANGELOG.md                # Release version history
├── app/                        # Frontend React SPA application
│   ├── public/                 # Static assets
│   ├── src/                    # React source code
│   │   ├── assets/             # CSS and media resources
│   │   ├── components/         # React UI components (Grid, PDFViewer, MediaPlayer, etc.)
│   │   ├── context/            # React Contexts (Auth, File operations, Theme)
│   │   ├── hooks/              # Custom hooks (e.g., useProgressiveFiles)
│   │   ├── services/           # API services (centralized client in `api.ts`)
│   │   ├── utils/              # Extension helpers (`fileExtensions.ts`) & utilities
│   │   ├── App.tsx             # Main layout & router
│   │   └── main.tsx            # Entry point
│   ├── package.json            # Node dependencies
│   ├── pnpm-lock.yaml          # Lockfile (pnpm required)
│   ├── tsconfig.json           # TypeScript configuration
│   └── vite.config.ts          # Vite bundler config
└── app/src-tauri/              # Headless Rust backend server
    ├── Cargo.toml              # Cargo dependencies (Axum, Actix, Grammers, Tokio)
    ├── src/
    │   ├── commands/           # REST API route handlers (auth, fs, files, search)
    │   ├── bandwidth.rs        # Bandwidth throttling & monitoring
    │   ├── lib.rs              # Router creation & state initialization
    │   ├── main.rs             # Server binary entry point (Axum + Actix threads)
    │   ├── models.rs           # Shared API request/response structs
    │   └── server.rs           # Actix Web streaming server implementation
```

---

## Build Targets & Development Commands

### Package Management Constraint
* **Package Manager**: Use **`pnpm`** exclusively for all frontend dependency installation and Node script execution in `app/`. Do not run `npm` or `yarn`.

### Frontend Commands (`app/`)
* **Install dependencies**: `cd app && pnpm install`
* **Development server**: `cd app && pnpm dev` (Runs Vite dev server on `http://localhost:1420`)
* **Type-check & build**: `cd app && pnpm build` (Compiles static assets into `app/dist`)
* **Preview build**: `cd app && pnpm preview`

### Backend Commands (`app/src-tauri/`)
* **Check compilation**: `cd app/src-tauri && cargo check`
* **Run server**: `cd app/src-tauri && cargo run` (Spawns Axum REST API on port `8550` and Actix media server on port `14201`)
* **Run unit tests**: `cd app/src-tauri && cargo test`
* **Build release binary**: `cd app/src-tauri && cargo build --release` (Generates binary at `app/src-tauri/target/release/app`)

### Docker Deployment
* **Build image**: `docker build -t telegram-drive .`
* **Run container**: `docker run -p 8550:8550 -v telegram_data:/app/data telegram-drive`

---

## Network Architecture, Ports, & Environment Variables

### Network Ports
* **`8550`**: Axum REST API & static web server (configurable via `PORT` environment variable).
* **`14201`**: Dedicated Actix Web server for streaming media files (audio/video).
* **`1420`**: Vite development server port during local UI development.

### Environment Variables
* `PORT`: Axum web server listening port (default: `8550`).
* `STATIC_DIR`: Path to compiled frontend dist directory (`app/dist`). When provided, Axum serves the SPA interface directly.
* `DATA_DIR`: Directory where `telegram.session` SQLite database, `session_config.json`, and `thumbnails/` cache are stored (defaults to system app data directory or `/app/data` in Docker).
* `RUST_LOG`: Logging directive level (e.g. `info`, `debug`).

---

## Low-Bandwidth & VPN Optimization Paradigms

The backend is specifically engineered for unreliable or high-latency VPN tunnels:

1. **Multi-DC Ping & Connection Fallback**: Attempts connections across Telegram DCs 1-5 to discover the most stable data center route rather than binding to a single static IP.
2. **Exponential Backoff & Flood Wait Handling**: Retries network requests with backoff on broken pipe / TCP reset errors, and transparently sleeps during Telegram `FLOOD_WAIT` rate limits.
3. **O(1) In-Memory Peer Lookup**: Caches Telegram dialog and channel peers in an in-memory `HashMap`, avoiding expensive round-trip peer iteration.
4. **Targeted Message Fetching**: Replaces linear message scanning with `get_messages_by_id` calls for direct file downloads and thumbnail extraction.
5. **Cursor-Based Progressive Pagination**: `cmd_get_files` supports `offset` and `limit` parameters, returning `FilePage` cursor objects so the frontend can load files progressively.
6. **Adaptive Frontend Network Polling**: Scales polling interval between 30s and 45s to minimize background VPN traffic.

---

## Conventions for AI Assistants

1. **Documentation Constraint**: Keep modifications restricted to high-level Markdown documentation files (e.g. `README.md`, `app/README.md`, `AGENTS.md`). Do **NOT** add or edit inline docstrings or comments in source code files (`.rs`, `.ts`, `.tsx`, `.js`) unless specifically requested.
2. **Central Frontend API Service**: All API calls from React should originate or be routed through `app/src/services/api.ts`.
3. **Central Rust Command Handlers**: API endpoints are registered in `app/src-tauri/src/lib.rs` and implemented in `app/src-tauri/src/commands/`.
4. **Verification**: Always run `cargo test` in `app/src-tauri` and `pnpm build` in `app` after making changes to verify compilation and test integrity.
