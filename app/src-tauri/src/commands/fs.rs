use grammers_client::types::{Media, Peer};
use grammers_client::InputMessage;
use grammers_tl_types as tl;
use serde::Deserialize;
use axum::{extract::{State, Query}, response::IntoResponse, Json, http::StatusCode};

use crate::commands::AppState;
use crate::models::{FolderMetadata, FileMetadata, FilePage};
use crate::commands::utils::{resolve_peer, ensure_cache_warm, map_error};
use crate::commands::retry::with_retry;

#[derive(Deserialize)]
pub struct CreateFolderPayload {
  pub name: String,
}

pub async fn cmd_create_folder(
  State(app_state): State<AppState>,
  Json(payload): Json<CreateFolderPayload>,
) -> impl IntoResponse {
  let state = &app_state.telegram;
  let name = payload.name;
  let client_opt = {
    state.client.lock().await.clone()
  };

  if client_opt.is_none() {
    let mock_id = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|d| d.as_secs() as i64)
      .unwrap_or_else(|_| chrono::Utc::now().timestamp());
    log::info!("[MOCK] Created folder '{}' with ID {}", name, mock_id);
    return (StatusCode::OK, Json(FolderMetadata {
      id: mock_id,
      name,
      parent_id: None,
    })).into_response();
  }

  let client = client_opt.unwrap();
  log::info!("Creating Telegram Channel: {}", name);

  let result = match client.invoke(&tl::functions::channels::CreateChannel {
    broadcast: true,
    megagroup: false,
    title: format!("{} [TD]", name),
    about: "Telegram Drive Storage Folder\n[telegram-drive-folder]".to_string(),
    geo_point: None,
    address: None,
    for_import: false,
    forum: false,
    ttl_period: None,
  }).await {
    Ok(res) => res,
    Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, map_error(e)).into_response(),
  };

  let (chat_id, access_hash) = match result {
    tl::enums::Updates::Updates(u) => {
      let chat = match u.chats.first() {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, "No chat in updates").into_response(),
      };
      match chat {
        tl::enums::Chat::Channel(c) => (c.id, c.access_hash.unwrap_or(0)),
        _ => return (StatusCode::BAD_REQUEST, "Created chat is not a channel").into_response(),
      }
    },
    _ => return (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected response (not Updates::Updates)").into_response(),
  };

  let _ = client.invoke(&tl::functions::messages::SetHistoryTtl {
    peer: tl::enums::InputPeer::Channel(tl::types::InputPeerChannel {
      channel_id: chat_id,
      access_hash,
    }),
    period: 0,
  }).await;

  (StatusCode::OK, Json(FolderMetadata {
    id: chat_id,
    name,
    parent_id: None,
  })).into_response()
}

#[derive(Deserialize)]
pub struct DeleteFolderPayload {
  pub folder_id: i64,
}

pub async fn cmd_delete_folder(
  State(app_state): State<AppState>,
  Json(payload): Json<DeleteFolderPayload>,
) -> impl IntoResponse {
  let state = &app_state.telegram;
  let folder_id = payload.folder_id;
  let client_opt = {
    state.client.lock().await.clone()
  };
  if client_opt.is_none() {
    log::info!("[MOCK] Deleted folder ID {}", folder_id);
    return (StatusCode::OK, Json(true)).into_response();
  }
  let client = client_opt.unwrap();
  log::info!("Deleting folder/channel: {}", folder_id);
  let peer = match resolve_peer(&client, Some(folder_id), state).await {
    Ok(p) => p,
    Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
  };
  let input_channel = match peer {
    Peer::Channel(c) => {
      let chan = &c.raw;
      let access_hash = match chan.access_hash {
        Some(h) => h,
        None => return (StatusCode::BAD_REQUEST, "No access hash for channel").into_response(),
      };
      tl::enums::InputChannel::Channel(tl::types::InputChannel {
        channel_id: chan.id,
        access_hash,
      })
    },
    _ => return (StatusCode::BAD_REQUEST, "Only channels (folders) can be deleted.").into_response(),
  };
  if let Err(e) = client.invoke(&tl::functions::channels::DeleteChannel {
    channel: input_channel,
  }).await {
    return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete channel: {}", e)).into_response();
  }
  state.peer_cache.lock().await.remove(&folder_id);
  (StatusCode::OK, Json(true)).into_response()
}

#[derive(Deserialize)]
pub struct UploadFilePayload {
  pub path: String,
  pub folder_id: Option<i64>,
}

pub async fn cmd_upload_file(
  State(app_state): State<AppState>,
  Json(payload): Json<UploadFilePayload>,
) -> impl IntoResponse {
  let state = &app_state.telegram;
  let bw_state = &app_state.bandwidth;
  let path = payload.path;
  let folder_id = payload.folder_id;
  let size = match std::fs::metadata(&path) {
    Ok(m) => m.len(),
    Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
  };

  if let Err(e) = bw_state.can_transfer(size) {
    return (StatusCode::FORBIDDEN, e).into_response();
  }

  let client_opt = {
    state.client.lock().await.clone()
  };

  if client_opt.is_none() {
    log::info!("[MOCK] Uploaded file {} to {:?}", path, folder_id);
    bw_state.add_up(size);
    return (StatusCode::OK, Json("Mock upload successful".to_string())).into_response();
  }

  let client = client_opt.unwrap();
  let path_clone = path.clone();
  let client_clone = client.clone();

  let uploaded_file = match with_retry(
    || {
      let p = path_clone.clone();
      let c = client_clone.clone();
      async move {
        c.upload_file(&p).await.map_err(map_error)
      }
    },
    2,
    2000,
  ).await {
    Ok(file) => file,
    Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
  };

  let message = InputMessage::new().text("").file(uploaded_file);
  let peer = match resolve_peer(&client, folder_id, state).await {
    Ok(p) => p,
    Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
  };
  if let Err(e) = client.send_message(&peer, message).await {
    return (StatusCode::INTERNAL_SERVER_ERROR, map_error(e)).into_response();
  }
  bw_state.add_up(size);
  (StatusCode::OK, Json("File uploaded successfully".to_string())).into_response()
}

#[derive(Deserialize)]
pub struct DeleteFilePayload {
  pub message_id: i32,
  pub folder_id: Option<i64>,
}

pub async fn cmd_delete_file(
  State(app_state): State<AppState>,
  Json(payload): Json<DeleteFilePayload>,
) -> impl IntoResponse {
  let state = &app_state.telegram;
  let message_id = payload.message_id;
  let folder_id = payload.folder_id;
  let client_opt = {
    state.client.lock().await.clone()
  };
  if client_opt.is_none() {
    log::info!("[MOCK] Deleted message {} from folder {:?}", message_id, folder_id);
    return (StatusCode::OK, Json(true)).into_response();
  }
  let client = client_opt.unwrap();
  let peer = match resolve_peer(&client, folder_id, state).await {
    Ok(p) => p,
    Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
  };
  if let Err(e) = client.delete_messages(&peer, &[message_id]).await {
    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
  }
  (StatusCode::OK, Json(true)).into_response()
}

#[derive(Deserialize)]
pub struct DownloadFilePayload {
  pub message_id: i32,
  pub save_path: String,
  pub folder_id: Option<i64>,
}

pub async fn cmd_download_file(
  State(app_state): State<AppState>,
  Json(payload): Json<DownloadFilePayload>,
) -> impl IntoResponse {
  let state = &app_state.telegram;
  let bw_state = &app_state.bandwidth;
  let message_id = payload.message_id;
  let save_path = payload.save_path;
  let folder_id = payload.folder_id;

  let client_opt = {
    state.client.lock().await.clone()
  };
  if client_opt.is_none() {
    log::info!("[MOCK] Downloaded message {} from {:?} to {}", message_id, folder_id, save_path);
    if let Err(e) = std::fs::write(&save_path, b"Mock Content") {
      return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }
    return (StatusCode::OK, Json("Download successful".to_string())).into_response();
  }
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
      let size = match &media {
        Media::Document(d) => d.size() as u64,
        Media::Photo(_) => 1024 * 1024,
        _ => 0,
      };

      if let Err(e) = bw_state.can_transfer(size) {
        return (StatusCode::FORBIDDEN, e).into_response();
      }

      let sp = save_path.clone();
      let c = client.clone();
      let m = media.clone();
      if let Err(e) = with_retry(
        || {
          let sp2 = sp.clone();
          let c2 = c.clone();
          let m2 = m.clone();
          async move {
            c2.download_media(&m2, &sp2).await.map_err(map_error)
          }
        },
        2,
        2000,
      ).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response();
      }

      bw_state.add_down(size);
      return (StatusCode::OK, Json("Download successful".to_string())).into_response();
    }
  }
  (StatusCode::NOT_FOUND, "Not found".to_string()).into_response()
}

#[derive(Deserialize)]
pub struct MoveFilesPayload {
  pub message_ids: Vec<i32>,
  pub source_folder_id: Option<i64>,
  pub target_folder_id: Option<i64>,
}

pub async fn cmd_move_files(
  State(app_state): State<AppState>,
  Json(payload): Json<MoveFilesPayload>,
) -> impl IntoResponse {
  let state = &app_state.telegram;
  let message_ids = payload.message_ids;
  let source_folder_id = payload.source_folder_id;
  let target_folder_id = payload.target_folder_id;

  if source_folder_id == target_folder_id {
    return (StatusCode::OK, Json(true)).into_response();
  }

  let client_opt = {
    state.client.lock().await.clone()
  };
  if client_opt.is_none() {
    log::info!("[MOCK] Moved msgs {:?} from {:?} to {:?}", message_ids, source_folder_id, target_folder_id);
    return (StatusCode::OK, Json(true)).into_response();
  }
  let client = client_opt.unwrap();

  ensure_cache_warm(&client, state).await;

  let source_peer = match resolve_peer(&client, source_folder_id, state).await {
    Ok(p) => p,
    Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
  };
  let target_peer = match resolve_peer(&client, target_folder_id, state).await {
    Ok(p) => p,
    Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
  };

  if let Err(e) = client.forward_messages(&target_peer, &message_ids, &source_peer).await {
    return (StatusCode::INTERNAL_SERVER_ERROR, format!("Forward failed: {}", e)).into_response();
  }
  if let Err(e) = client.delete_messages(&source_peer, &message_ids).await {
    return (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete original failed: {}", e)).into_response();
  }
  (StatusCode::OK, Json(true)).into_response()
}

#[derive(Deserialize)]
pub struct GetFilesParams {
  pub folder_id: Option<i64>,
  pub offset: Option<i32>,
  pub limit: Option<i32>,
}

pub async fn cmd_get_files(
  State(app_state): State<AppState>,
  Query(params): Query<GetFilesParams>,
) -> impl IntoResponse {
  let state = &app_state.telegram;
  let folder_id = params.folder_id;
  let offset = params.offset.unwrap_or(0);
  let limit = params.limit.unwrap_or(50).min(500);

  let client_opt = { state.client.lock().await.clone() };
  if client_opt.is_none() {
    log::info!("[MOCK] Returning mock files for folder {:?}", folder_id);
    return (StatusCode::OK, Json(FilePage {
      files: Vec::new(),
      has_more: false,
      next_offset: 0,
      total_fetched: 0,
    })).into_response();
  }
  let client = client_opt.unwrap();
  let peer = match resolve_peer(&client, folder_id, state).await {
    Ok(p) => p,
    Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
  };
  let mut msgs = client.iter_messages(&peer);

  let mut skipped = 0;
  while skipped < offset {
    match msgs.next().await {
      Ok(Some(_)) => skipped += 1,
      Ok(None) => {
        return (StatusCode::OK, Json(FilePage {
          files: Vec::new(),
          has_more: false,
          next_offset: offset,
          total_fetched: 0,
        })).into_response();
      },
      Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
  }

  let mut files = Vec::new();
  let mut messages_seen = 0;
  let mut hit_end = false;

  while (files.len() as i32) < limit {
    match msgs.next().await {
      Ok(Some(msg)) => {
        messages_seen += 1;
        if let Some(doc) = msg.media() {
          let (name, size, mime, ext) = match doc {
            Media::Document(d) => {
              let n = d.name().to_string();
              let s = d.size();
              let m = d.mime_type().map(|s| s.to_string());
              let e = std::path::Path::new(&n)
                .extension()
                .map(|os| os.to_str().unwrap_or("").to_string());
              (n, s, m, e)
            },
            Media::Photo(_) => ("Photo.jpg".to_string(), 0, Some("image/jpeg".into()), Some("jpg".into())),
            _ => ("Unknown".to_string(), 0, None, None),
          };

          files.push(FileMetadata {
            id: msg.id() as i64,
            folder_id,
            name,
            size: size as u64,
            mime_type: mime,
            file_ext: ext,
            created_at: msg.date().to_string(),
            icon_type: "file".into(),
          });
        }
      },
      Ok(None) => { hit_end = true; break; },
      Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
  }

  let fetched_count = files.len() as i32;
  (StatusCode::OK, Json(FilePage {
    files,
    has_more: !hit_end,
    next_offset: offset + messages_seen,
    total_fetched: fetched_count,
  })).into_response()
}

fn extract_file_from_message(m: &tl::types::Message) -> Option<FileMetadata> {
  let media = m.media.as_ref()?;
  let doc_media = match media {
    tl::enums::MessageMedia::Document(d) => d,
    _ => return None,
  };
  let doc = match doc_media.document.as_ref()? {
    tl::enums::Document::Document(doc) => doc,
    _ => return None,
  };

  let name = doc.attributes.iter().find_map(|a| match a {
    tl::enums::DocumentAttribute::Filename(f) => Some(f.file_name.clone()),
    _ => None,
  }).unwrap_or_else(|| "Unknown".to_string());

  let size = doc.size as u64;
  let mime = doc.mime_type.clone();
  let ext = std::path::Path::new(&name)
    .extension()
    .map(|os| os.to_str().unwrap_or("").to_string());

  let folder_id = match &m.peer_id {
    tl::enums::Peer::Channel(c) => Some(c.channel_id),
    tl::enums::Peer::User(u) => Some(u.user_id),
    tl::enums::Peer::Chat(c) => Some(c.chat_id),
  };

  Some(FileMetadata {
    id: m.id as i64,
    folder_id,
    name,
    size,
    mime_type: Some(mime),
    file_ext: ext,
    created_at: m.date.to_string(),
    icon_type: "file".into(),
  })
}

#[derive(Deserialize)]
pub struct SearchGlobalParams {
  pub query: String,
}

pub async fn cmd_search_global(
  State(app_state): State<AppState>,
  Query(params): Query<SearchGlobalParams>,
) -> impl IntoResponse {
  let state = &app_state.telegram;
  let query = params.query;
  let client_opt = { state.client.lock().await.clone() };
  if client_opt.is_none() {
    return (StatusCode::OK, Json(Vec::<FileMetadata>::new())).into_response();
  }
  let client = client_opt.unwrap();
  let mut files = Vec::new();
  log::info!("Searching global for: {}", query);

  let result = match client.invoke(&tl::functions::messages::SearchGlobal {
    q: query,
    filter: tl::enums::MessagesFilter::InputMessagesFilterDocument,
    min_date: 0,
    max_date: 0,
    offset_rate: 0,
    offset_peer: tl::enums::InputPeer::Empty,
    offset_id: 0,
    limit: 50,
    folder_id: None,
    broadcasts_only: false,
    groups_only: false,
    users_only: false,
  }).await {
    Ok(res) => res,
    Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, map_error(e)).into_response(),
  };

  let messages = match result {
    tl::enums::messages::Messages::Messages(msgs) => msgs.messages,
    tl::enums::messages::Messages::Slice(msgs) => msgs.messages,
    _ => Vec::new(),
  };

  for msg in messages {
    if let tl::enums::Message::Message(m) = msg {
      if let Some(file) = extract_file_from_message(&m) {
        files.push(file);
      }
    }
  }

  (StatusCode::OK, Json(files)).into_response()
}

pub async fn cmd_scan_folders(
  State(app_state): State<AppState>,
) -> impl IntoResponse {
  let state = &app_state.telegram;
  let client_opt = { state.client.lock().await.clone() };
  if client_opt.is_none() {
    return (StatusCode::OK, Json(Vec::<FolderMetadata>::new())).into_response();
  }
  let client = client_opt.unwrap();
  let mut folders = Vec::new();
  let mut dialogs = client.iter_dialogs();

  log::info!("Starting Folder Scan...");

  while let Some(dialog) = match dialogs.next().await {
    Ok(d) => d,
    Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
  } {
    let peer_id = match &dialog.peer {
      Peer::Channel(c) => Some(c.raw.id),
      Peer::User(u) => Some(u.raw.id()),
      _ => None,
    };

    if let Some(id) = peer_id {
      state.peer_cache.lock().await.insert(id, dialog.peer.clone());
    }

    match &dialog.peer {
      Peer::Channel(c) => {
        let id = c.raw.id;
        let name = c.raw.title.clone();
        let access_hash = c.raw.access_hash.unwrap_or(0);
        log::debug!("[SCAN] Processing Channel: '{}' (ID: {})", name, id);

        if name.to_lowercase().contains("[td]") {
          log::info!(" -> MATCH via Title: {}", name);
          let display_name = name
            .replace(" [TD]", "")
            .replace(" [td]", "")
            .replace("[TD]", "")
            .replace("[td]", "")
            .trim()
            .to_string();
          folders.push(FolderMetadata {
            id,
            name: display_name,
            parent_id: None,
          });
          continue;
        }

        let input_chan = tl::enums::InputChannel::Channel(tl::types::InputChannel {
          channel_id: c.raw.id,
          access_hash,
        });
        match client.invoke(&tl::functions::channels::GetFullChannel {
          channel: input_chan,
        }).await {
          Ok(tl::enums::messages::ChatFull::Full(f)) => {
            if let tl::enums::ChatFull::Full(cf) = f.full_chat {
              if cf.about.contains("[telegram-drive-folder]") {
                log::info!(" -> MATCH via About: {}", name);
                folders.push(FolderMetadata {
                  id,
                  name: name.clone(),
                  parent_id: None,
                });
              }
            }
          },
          Err(e) => log::warn!(" -> Failed to get full info: {}", e),
        }
      },
      peer => {
        log::debug!("[SCAN] Skipped Peer: {:?}", peer);
      }
    }
  }

  log::info!("Scan complete. Found {} folders.", folders.len());
  (StatusCode::OK, Json(folders)).into_response()
}
