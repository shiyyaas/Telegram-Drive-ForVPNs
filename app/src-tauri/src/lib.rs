pub mod models;
pub mod commands;
pub mod bandwidth;
pub mod server;

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

use commands::{TelegramState, AppState};
use bandwidth::BandwidthManager;

pub fn create_router(telegram_state: TelegramState, bw_manager: Arc<BandwidthManager>) -> Router {
    let app_state = AppState {
        telegram: telegram_state,
        bandwidth: bw_manager,
    };

    let api_router = Router::new()
        .route("/connect", post(commands::cmd_connect))
        .route("/check-connection", get(commands::cmd_check_connection))
        .route("/logout", post(commands::cmd_logout))
        .route("/auth/request-code", post(commands::cmd_auth_request_code))
        .route("/auth/sign-in", post(commands::cmd_auth_sign_in))
        .route("/auth/check-password", post(commands::cmd_auth_check_password))
        .route("/set-proxy", post(commands::cmd_set_proxy))
        .route("/is-network-available", get(commands::network::cmd_is_network_available))
        .route("/folders", post(commands::cmd_create_folder).get(commands::cmd_scan_folders))
        .route("/folders/delete", post(commands::cmd_delete_folder))
        .route("/files", get(commands::cmd_get_files))
        .route("/files/upload", post(commands::cmd_upload_file))
        .route("/files/delete", post(commands::cmd_delete_file))
        .route("/files/download", post(commands::cmd_download_file))
        .route("/files/move", post(commands::cmd_move_files))
        .route("/files/search", get(commands::cmd_search_global))
        .route("/preview", get(commands::cmd_get_preview))
        .route("/thumbnail", get(commands::cmd_get_thumbnail))
        .route("/clean-cache", post(commands::cmd_clean_cache))
        .route("/bandwidth", get(commands::cmd_get_bandwidth))
        .route("/stream-port", get(commands::cmd_get_stream_port))
        .route("/log", post(commands::cmd_log))
        .with_state(app_state);

    let dist_path = std::env::var("STATIC_DIR").unwrap_or_else(|_| "../app/dist".to_string());
    let index_path = format!("{}/index.html", dist_path);

    let serve_dir = ServeDir::new(&dist_path)
        .not_found_service(ServeFile::new(&index_path));

    Router::new()
        .nest("/api", api_router)
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
}

pub fn init_telegram_state() -> TelegramState {
    TelegramState {
        client: Arc::new(Mutex::new(None)),
        login_token: Arc::new(Mutex::new(None)),
        password_token: Arc::new(Mutex::new(None)),
        api_id: Arc::new(Mutex::new(None)),
        runner_shutdown: Arc::new(std::sync::Mutex::new(None)),
        runner_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        peer_cache: Arc::new(Mutex::new(HashMap::new())),
        proxy_url: Arc::new(Mutex::new(None)),
    }
}
