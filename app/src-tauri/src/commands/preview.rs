use grammers_client::types::Media;
use base64::{Engine as _, engine::general_purpose};
use serde::Deserialize;
use axum::{extract::{State, Query}, response::IntoResponse, Json, http::StatusCode};

use crate::commands::AppState;
use crate::commands::utils::resolve_peer;

#[derive(Deserialize)]
pub struct GetPreviewParams {
    pub message_id: i32,
    pub folder_id: Option<i64>,
}

pub async fn cmd_get_preview(
    State(app_state): State<AppState>,
    Query(params): Query<GetPreviewParams>,
) -> impl IntoResponse {
    let state = &app_state.telegram;
    let bw_state = &app_state.bandwidth;
    let message_id = params.message_id;
    let folder_id = params.folder_id;

    let app_data_dir = std::env::var("DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("data"));
    
    let cache_dir = app_data_dir.join("previews");
    if !cache_dir.exists() { let _ = std::fs::create_dir_all(&cache_dir); }
    log::info!("Using preview cache dir: {:?}", cache_dir);
    log::info!("Preview Request: msg_id={}", message_id);

    let client_opt = { state.client.lock().await.clone() };
    if client_opt.is_none() { return (StatusCode::OK, Json("".to_string())).into_response(); }
    let client = client_opt.unwrap();
    
    let peer = match resolve_peer(&client, folder_id, state).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    let messages = match client.get_messages_by_id(&peer, &[message_id]).await {
        Ok(m) => m,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let target_message = messages.into_iter().next().flatten();
    
    if let Some(msg) = target_message {
        if let Some(media) = msg.media() {
             let ext = match &media {
                 Media::Document(d) => {
                     let mut e = std::path::Path::new(d.name()).extension().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                     if e.is_empty() {
                         if let Some(mime) = d.mime_type() {
                              e = match mime {
                                  "image/jpeg" => "jpg".to_string(),
                                  "image/png" => "png".to_string(),
                                  "video/mp4" => "mp4".to_string(),
                                  _ => "bin".to_string(),
                              };
                         } else {
                             e = "bin".to_string();
                         }
                     }
                     e
                 },
                 Media::Photo(_) => "jpg".to_string(),
                 _ => "bin".to_string(),
             };
             
             let save_path = cache_dir.join(format!("{}.{}", message_id, ext));
             let save_path_str = save_path.to_string_lossy().to_string();
             
             let file_ready = if save_path.exists() {
                 log::info!("File ({}) exists in cache.", message_id);
                 true
             } else {
                 let size = match &media {
                    Media::Document(d) => d.size() as u64,
                    Media::Photo(_) => 1024 * 1024,
                    _ => 0,
                };
                
                log::info!("Downloading preview... Size: {}", size);
                if let Err(e) = bw_state.can_transfer(size) {
                    log::warn!("Bandwidth limit hit for preview: {}", e);
                    false
                } else {
                    match client.download_media(&media, &save_path_str).await {
                        Ok(_) => {
                            log::info!("Preview download complete.");
                            bw_state.add_down(size);
                            true
                        },
                        Err(e) => {
                            log::error!("Preview Download Error: {}", e);
                            false
                        }
                    }
                }
             };

             if file_ready {
                 let lower_ext = ext.to_lowercase();
                 if ["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg"].contains(&lower_ext.as_str()) {
                     log::info!("Converting image to Base64...");
                     match std::fs::read(&save_path) {
                         Ok(bytes) => {
                             let b64 = general_purpose::STANDARD.encode(&bytes);
                             let mime = match lower_ext.as_str() {
                                 "png" => "image/png",
                                 "gif" => "image/gif",
                                 "webp" => "image/webp",
                                 "bmp" => "image/bmp",
                                 "svg" => "image/svg+xml",
                                 _ => "image/jpeg",
                             };
                             return (StatusCode::OK, Json(format!("data:{};base64,{}", mime, b64))).into_response();
                         },
                         Err(e) => {
                             log::error!("Failed to read file for base64: {}", e);
                             return (StatusCode::OK, Json(save_path_str)).into_response();
                         }
                     }
                 }
                 log::info!("Returning path preview: {}", save_path_str);
                 return (StatusCode::OK, Json(save_path_str)).into_response();
             }
        }
    }

    (StatusCode::INTERNAL_SERVER_ERROR, "File not found or failed to download".to_string()).into_response()
}

pub async fn cmd_clean_cache() -> impl IntoResponse {
    let app_data_dir = std::env::var("DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("data"));
    let cache_dir = app_data_dir.join("previews");
    if cache_dir.exists() {
         let _ = std::fs::remove_dir_all(cache_dir);
    }
    (StatusCode::OK, Json(())).into_response()
}

#[derive(Deserialize)]
pub struct GetThumbnailParams {
    pub message_id: i32,
    pub folder_id: Option<i64>,
}

pub async fn cmd_get_thumbnail(
    State(app_state): State<AppState>,
    Query(params): Query<GetThumbnailParams>,
) -> impl IntoResponse {
    let state = &app_state.telegram;
    let message_id = params.message_id;
    let folder_id = params.folder_id;

    let app_data_dir = std::env::var("DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("data"));
    let cache_dir = app_data_dir.join("thumbnails");
    if !cache_dir.exists() { let _ = std::fs::create_dir_all(&cache_dir); }
    
    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("{}.", message_id)) {
                if let Ok(bytes) = std::fs::read(entry.path()) {
                    let ext = name.rsplit('.').next().unwrap_or("jpg");
                    let mime = match ext {
                        "png" => "image/png",
                        "gif" => "image/gif", 
                        "webp" => "image/webp",
                        _ => "image/jpeg",
                    };
                    let b64 = general_purpose::STANDARD.encode(&bytes);
                    return (StatusCode::OK, Json(format!("data:{};base64,{}", mime, b64))).into_response();
                }
            }
        }
    }
    
    let client_opt = { state.client.lock().await.clone() };
    if client_opt.is_none() { return (StatusCode::OK, Json("".to_string())).into_response(); }
    let client = client_opt.unwrap();
    
    let peer = match resolve_peer(&client, folder_id, state).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    let messages = match client.get_messages_by_id(&peer, &[message_id]).await {
        Ok(m) => m,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if let Some(Some(m)) = messages.into_iter().next() {
        if let Some(media) = m.media() {
            let (is_image, ext) = match &media {
                Media::Photo(_) => (true, "jpg".to_string()),
                Media::Document(d) => {
                    let mime = d.mime_type().unwrap_or("");
                    if mime.starts_with("image/") {
                        let e = match mime {
                            "image/png" => "png",
                            "image/gif" => "gif",
                            "image/webp" => "webp",
                            _ => "jpg",
                        };
                        (true, e.to_string())
                    } else {
                        return (StatusCode::OK, Json("".to_string())).into_response();
                    }
                },
                _ => return (StatusCode::OK, Json("".to_string())).into_response(),
            };
            
            if is_image {
                let save_path = cache_dir.join(format!("{}.{}", message_id, ext));
                let save_path_str = save_path.to_string_lossy().to_string();
                
                if client.download_media(&media, &save_path_str).await.is_ok() {
                    if let Ok(bytes) = std::fs::read(&save_path) {
                        let mime = match ext.as_str() {
                            "png" => "image/png",
                            "gif" => "image/gif",
                            "webp" => "image/webp",
                            _ => "image/jpeg",
                        };
                        let b64 = general_purpose::STANDARD.encode(&bytes);
                        return (StatusCode::OK, Json(format!("data:{};base64,{}", mime, b64))).into_response();
                    }
                }
            }
        }
    }
    
    (StatusCode::OK, Json("".to_string())).into_response()
}
