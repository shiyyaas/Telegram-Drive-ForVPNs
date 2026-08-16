use grammers_client::Client;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use grammers_mtsender::{SenderPool, ConnectionParams};
use grammers_session::storages::SqliteSession;
use tokio::sync::oneshot;
use tokio::time::Duration;
use serde::Deserialize;
use axum::{extract::State, response::IntoResponse, Json, http::StatusCode};

use crate::commands::AppState;
use crate::TelegramState;
use crate::models::AuthResult;
use crate::commands::utils::map_error;
use grammers_client::SignInError;

pub async fn ensure_client_initialized(
    state: &TelegramState,
    api_id: i32,
) -> Result<Client, String> {
    let mut client_guard = state.client.lock().await;

    if let Some(client) = client_guard.as_ref() {
        return Ok(client.clone());
    }

    let should_wait = {
        let mut shutdown_guard = state.runner_shutdown.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(shutdown_tx) = shutdown_guard.take() {
            log::info!("Signaling old runner to shutdown...");
            let _ = shutdown_tx.send(());
            true
        } else {
            false
        }
    };
    if should_wait {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let runner_num = state.runner_count.fetch_add(1, Ordering::SeqCst) + 1;
    log::info!("Initializing Telegram Client #{} with API ID: {}", runner_num, api_id);
    
    let app_data_dir = std::env::var("DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("data"));
        
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }
    
    let session_path = app_data_dir.join("telegram.session");
    let session_path_str = session_path.to_string_lossy().to_string();
    log::info!("Opening session at: {}", session_path_str);
    
    let session = match SqliteSession::open(&session_path_str).map_err(|e| e.to_string()) {
        Ok(s) => s,
        Err(_) => {
            log::warn!("Session file corrupted or invalid. Recreating...");
            let _ = std::fs::remove_file(&session_path);
            let _ = std::fs::remove_file(format!("{}-wal", session_path_str));
            let _ = std::fs::remove_file(format!("{}-shm", session_path_str));
            
            SqliteSession::open(&session_path_str)
                .map_err(|e| format!("Failed to open session after recreation: {}", e))?
        }
    };
        
    let session = Arc::new(session);

    let proxy = state.proxy_url.lock().await.clone();
    let connection_params = ConnectionParams {
        proxy_url: proxy.clone(),
        ..Default::default()
    };
    if let Some(ref p) = proxy {
        log::info!("Using SOCKS5 proxy: {}", p);
    }

    let pool = SenderPool::with_configuration(session, api_id, connection_params);
    let client = Client::new(&pool);
    
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    *state.runner_shutdown.lock().unwrap_or_else(|e| e.into_inner()) = Some(shutdown_tx);
    
    let SenderPool { runner, .. } = pool;
    tokio::spawn(async move {
        tokio::select! {
            _ = runner.run() => {
                log::info!("Runner #{} exited normally", runner_num);
            }
            _ = shutdown_rx => {
                log::info!("Runner #{} shutdown requested, exiting", runner_num);
            }
        }
    });
    
    *client_guard = Some(client.clone());
    Ok(client)
}

#[derive(Deserialize)]
pub struct ConnectPayload {
    pub api_id: i32,
}

pub async fn cmd_connect(
    State(app_state): State<AppState>,
    Json(payload): Json<ConnectPayload>,
) -> impl IntoResponse {
    let state = &app_state.telegram;
    *state.api_id.lock().await = Some(payload.api_id);
    match ensure_client_initialized(state, payload.api_id).await {
        Ok(_) => (StatusCode::OK, Json(true)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn cmd_check_connection(
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let state = &app_state.telegram;
    let client_msg_opt = {
        let guard = state.client.lock().await;
        guard.as_ref().cloned()
    };

    if let Some(client) = client_msg_opt {
        for attempt in 1..=2 {
            if client.get_me().await.is_ok() {
                return (StatusCode::OK, Json(true)).into_response();
            }
            if attempt < 2 {
                log::warn!("Connection check failed (attempt {}). Retrying in 3s...", attempt);
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
        log::warn!("Connection check failed after retries. Attempting reconnect...");
    } else {
        log::warn!("Connection check: No client found. Checking for saved API ID...");
    }

    let api_id_opt = *state.api_id.lock().await;
    if let Some(api_id) = api_id_opt {
        *state.client.lock().await = None;
        
        match ensure_client_initialized(state, api_id).await {
            Ok(c) => {
                if c.get_me().await.is_ok() {
                    log::info!("Auto-reconnect successful.");
                    return (StatusCode::OK, Json(true)).into_response();
                } else {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Reconnect succeeded but ping failed.").into_response();
                }
            },
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Auto-reconnect failed: {}", e)).into_response()
        }
    }

    (StatusCode::OK, Json(false)).into_response()
}

pub async fn cmd_logout(
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let state = &app_state.telegram;
    log::info!("Logging out...");
    
    {
        let mut shutdown_guard = state.runner_shutdown.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(shutdown_tx) = shutdown_guard.take() {
            log::info!("Signaling runner shutdown for logout...");
            let _ = shutdown_tx.send(());
        }
    }
    
    let client_opt = { state.client.lock().await.clone() };
    if let Some(client) = client_opt {
        let _ = client.sign_out().await; 
    }

    *state.client.lock().await = None;
    *state.login_token.lock().await = None;
    *state.password_token.lock().await = None;
    *state.api_id.lock().await = None;
    state.peer_cache.lock().await.clear();

    let app_data_dir = std::env::var("DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("data"));
    let session_path = app_data_dir.join("telegram.session");
    let _ = std::fs::remove_file(session_path);
    let _ = std::fs::remove_file(app_data_dir.join("telegram.session-wal"));
    let _ = std::fs::remove_file(app_data_dir.join("telegram.session-shm"));

    log::info!("Logout complete. Runner count: {}", state.runner_count.load(Ordering::SeqCst));
    (StatusCode::OK, Json(true)).into_response()
}

#[derive(Deserialize)]
pub struct AuthRequestCodePayload {
    pub phone: String,
    pub api_id: i32,
    pub api_hash: String,
}

pub async fn cmd_auth_request_code(
    State(app_state): State<AppState>,
    Json(payload): Json<AuthRequestCodePayload>,
) -> impl IntoResponse {
    let state = &app_state.telegram;
    if payload.api_hash.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "API Hash cannot be empty.").into_response();
    }

    *state.api_id.lock().await = Some(payload.api_id);

    let client_handle = match ensure_client_initialized(state, payload.api_id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };
    
    log::info!("Requesting code for {}", payload.phone);
    
    let mut last_error = String::new();
    
    for i in 1..=2 {
        match client_handle.request_login_code(&payload.phone, &payload.api_hash).await {
            Ok(token) => {
                let mut token_guard = state.login_token.lock().await;
                *token_guard = Some(token);
                return (StatusCode::OK, Json("code_sent".to_string())).into_response();
            },
            Err(e) => {
                let err_msg = e.to_string();
                log::warn!("Error requesting code (Attempt {}): {}", i, err_msg);
                
                if err_msg.contains("AUTH_RESTART") || err_msg.contains("500") {
                    log::info!("AUTH_RESTART error detected. Retrying...");
                    last_error = err_msg;
                    continue;
                }
                
                return (StatusCode::INTERNAL_SERVER_ERROR, map_error(e)).into_response();
            }
        }
    }

    (StatusCode::INTERNAL_SERVER_ERROR, format!("Telegram Error after retry: {}", last_error)).into_response()
}

#[derive(Deserialize)]
pub struct AuthSignInPayload {
    pub code: String,
}

pub async fn cmd_auth_sign_in(
    State(app_state): State<AppState>,
    Json(payload): Json<AuthSignInPayload>,
) -> impl IntoResponse {
    let state = &app_state.telegram;
    log::info!("Signing in with code...");
    
    let client = {
        let guard = state.client.lock().await;
        match guard.as_ref() {
            Some(c) => c.clone(),
            None => return (StatusCode::BAD_REQUEST, "Client not initialized").into_response(),
        }
    };

    let token_guard = state.login_token.lock().await;
    let login_token = match token_guard.as_ref() {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST, "No login session found (restart flow)").into_response(),
    };

    match client.sign_in(login_token, &payload.code).await {
        Ok(_user) => {
             log::info!("Successfully logged in.");
             (StatusCode::OK, Json(AuthResult {
                success: true,
                next_step: Some("dashboard".to_string()),
                error: None,
            })).into_response()
        }
        Err(SignInError::PasswordRequired(token)) => {
            let mut pw_guard = state.password_token.lock().await;
            *pw_guard = Some(token);

            (StatusCode::OK, Json(AuthResult {
                success: false,
                next_step: Some("password".to_string()),
                error: None,
            })).into_response()
        }
        Err(e) => {
           log::error!("Sign in error: {}", e);
           (StatusCode::INTERNAL_SERVER_ERROR, format!("Sign in failed: {}", e)).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct SetProxyPayload {
    pub proxy_url: Option<String>,
}

pub async fn cmd_set_proxy(
    State(app_state): State<AppState>,
    Json(payload): Json<SetProxyPayload>,
) -> impl IntoResponse {
    let state = &app_state.telegram;
    let cleaned = payload.proxy_url.filter(|s| !s.trim().is_empty());
    if let Some(ref url) = cleaned {
        log::info!("Proxy configured: {}", url);
    } else {
        log::info!("Proxy cleared (direct connection)");
    }
    *state.proxy_url.lock().await = cleaned;
    (StatusCode::OK, Json(true)).into_response()
}

#[derive(Deserialize)]
pub struct AuthCheckPasswordPayload {
    pub password: String,
}

pub async fn cmd_auth_check_password(
    State(app_state): State<AppState>,
    Json(payload): Json<AuthCheckPasswordPayload>,
) -> impl IntoResponse {
    let state = &app_state.telegram;
    let client = {
        let guard = state.client.lock().await;
        match guard.as_ref() {
            Some(c) => c.clone(),
            None => return (StatusCode::BAD_REQUEST, "Client not initialized").into_response(),
        }
    };
    
    let mut pw_guard = state.password_token.lock().await;
    let pw_token = match pw_guard.take() {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST, "No password session found").into_response(),
    };

    match client.check_password(pw_token, payload.password.as_str()).await {
        Ok(_user) => {
             log::info!("2FA Success.");
             (StatusCode::OK, Json(AuthResult {
                success: true,
                next_step: Some("dashboard".to_string()),
                error: None,
            })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("2FA Failed: {}", e)).into_response()
    }
}
