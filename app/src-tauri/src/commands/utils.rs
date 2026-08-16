use grammers_client::Client;
use grammers_client::types::Peer;
use axum::extract::State;
use crate::commands::AppState;
use crate::TelegramState;

pub async fn resolve_peer(client: &Client, folder_id: Option<i64>, state: &TelegramState) -> Result<Peer, String> {
    if let Some(fid) = folder_id {
        {
            let cache = state.peer_cache.lock().await;
            if let Some(cached_peer) = cache.get(&fid) {
                log::debug!("Peer cache HIT for folder {}", fid);
                return Ok(cached_peer.clone());
            }
        }

        log::debug!("Peer cache MISS for folder {}. Iterating dialogs...", fid);

        let mut dialogs = client.iter_dialogs();
        while let Some(dialog) = dialogs.next().await.map_err(|e| e.to_string())? {
            let peer_id = match &dialog.peer {
                Peer::Channel(c) => Some(c.raw.id),
                Peer::User(u) => Some(u.raw.id()),
                _ => None,
            };

            if let Some(id) = peer_id {
                let mut cache = state.peer_cache.lock().await;
                cache.insert(id, dialog.peer.clone());

                if id == fid {
                    return Ok(dialog.peer.clone());
                }
            }
        }
        Err(format!("Folder/Chat {} not found", fid))
    } else {
        match client.get_me().await {
            Ok(me) => Ok(Peer::User(me)),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub async fn cmd_get_stream_port() -> axum::response::Json<u16> {
    axum::response::Json(crate::server::STREAMING_PORT)
}

pub async fn ensure_cache_warm(client: &Client, state: &TelegramState) {
    {
        let cache = state.peer_cache.lock().await;
        if !cache.is_empty() {
            log::debug!("Peer cache already warm ({} entries)", cache.len());
            return;
        }
    }
    log::info!("Peer cache cold — pre-warming from dialog list...");
    let mut dialogs = client.iter_dialogs();
    while let Ok(Some(dialog)) = dialogs.next().await {
        let peer_id = match &dialog.peer {
            Peer::Channel(c) => Some(c.raw.id),
            Peer::User(u) => Some(u.raw.id()),
            _ => None,
        };
        if let Some(id) = peer_id {
            state.peer_cache.lock().await.insert(id, dialog.peer.clone());
        }
    }
    log::info!("Peer cache warmed ({} entries)", state.peer_cache.lock().await.len());
}

#[derive(serde::Deserialize)]
pub struct LogPayload {
    pub message: String,
}

pub async fn cmd_log(axum::extract::Json(payload): axum::extract::Json<LogPayload>) {
    log::info!("[FRONTEND] {}", payload.message);
}

pub async fn cmd_get_bandwidth(
    State(app_state): State<AppState>,
) -> axum::response::Json<crate::bandwidth::BandwidthStats> {
    axum::response::Json(app_state.bandwidth.get_stats())
}

pub fn map_error(e: impl std::fmt::Display) -> String {
    let err_str = e.to_string();
    if err_str.contains("FLOOD_WAIT") {
        if let Some(start) = err_str.find("(value: ") {
             let rest = &err_str[start + 8..];
             if let Some(end) = rest.find(')') {
                 if let Ok(seconds) = rest[..end].parse::<i64>() {
                     return format!("FLOOD_WAIT_{}", seconds);
                 }
             }
        }
        return "FLOOD_WAIT_60".to_string();
    }
    err_str
}
