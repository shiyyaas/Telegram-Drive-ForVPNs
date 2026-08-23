# Telegram Drive Frontend Application

This directory contains the web frontend for **Telegram Drive (Optimized for VPNs)**, built with **React 19**, **TypeScript**, **Vite**, **Tailwind CSS**, and **TanStack Query / Virtual**.

## Key Features & Components

* **Virtual Scrolling Grid/List**: Uses `@tanstack/react-virtual` for high-performance rendering of folders containing thousands of files.
* **Progressive Loading**: Interacts with cursor-paginated backend APIs to load files in responsive chunks on high-latency connections.
* **PDF Preview**: Built-in PDF reader powered by `pdfjs-dist` with page navigation, zoom control, and rotation.
* **Media Streaming Player**: Streams audio and video content directly from the Actix streaming server.
* **Centralized API Client**: Interacts with the Rust Axum backend (`http://localhost:8550/api` in development or `/api` in production).

## Available Scripts

Always use `pnpm` for package management and running scripts in this workspace:

| Command | Description |
|---------|-------------|
| `pnpm dev` | Starts the Vite development server on `http://localhost:1420` with HMR |
| `pnpm build` | Type-checks with `tsc` and compiles production assets into `dist/` |
| `pnpm preview` | Locally previews the production build output from `dist/` |

## Project Structure

```
app/
├── public/              # Static public assets
├── src/
│   ├── assets/          # Icons, logos, and stylesheets
│   ├── components/      # UI components (Grid, List, PDFViewer, MediaPlayer, etc.)
│   ├── context/         # React Contexts (Auth, File operations, Theme)
│   ├── hooks/           # Custom React hooks (useProgressiveFiles, etc.)
│   ├── services/        # Centralized API service client (`api.ts`)
│   ├── utils/           # Helper utilities and shared file extension definitions
│   ├── App.tsx          # Main application component
│   └── main.tsx         # Entry point
├── index.html           # HTML template
├── package.json         # Package configuration and dependencies
├── tsconfig.json        # TypeScript configuration
└── vite.config.ts       # Vite bundler configuration
```

## Backend API Client Integration

The frontend API client is defined in `src/services/api.ts`.
* In development, requests target `http://localhost:8550/api`.
* In production (when served directly by the Axum server), requests target relative `/api`.
* Media streaming URLs point to port `14201` (`http://localhost:14201/stream/...`).
