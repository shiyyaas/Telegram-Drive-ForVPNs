use grammers_client::Client;
use grammers_client::types::{LoginToken, PasswordToken, Peer};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

use crate::bandwidth::BandwidthManager;

#[derive(Clone)]
pub struct AppState {
    pub telegram: TelegramState,
    pub bandwidth: Arc<BandwidthManager>,
}

#[derive(Clone)]
pub struct TelegramState {
    pub client: Arc<Mutex<Option<Client>>>,
    pub login_token: Arc<Mutex<Option<LoginToken>>>,
    pub password_token: Arc<Mutex<Option<PasswordToken>>>,
    pub api_id: Arc<Mutex<Option<i32>>>,
    pub runner_shutdown: Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    pub runner_count: Arc<std::sync::atomic::AtomicU32>,
    pub peer_cache: Arc<Mutex<HashMap<i64, Peer>>>,
    pub proxy_url: Arc<Mutex<Option<String>>>,
}

pub mod auth;
pub mod fs;
pub mod preview;
pub mod utils;
pub mod network;
pub mod retry;

pub use auth::*;
pub use fs::*;
pub use preview::*;
pub use utils::*;
pub use network::*;
