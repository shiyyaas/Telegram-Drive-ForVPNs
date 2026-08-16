use std::net::TcpStream;
use std::time::Duration;
use axum::{response::IntoResponse, Json, http::StatusCode};

pub async fn cmd_is_network_available() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(|| {
        let dc_endpoints = [
            "149.154.175.53:443",   // DC1
            "149.154.167.50:443",   // DC2
            "149.154.175.100:443",  // DC3
            "149.154.167.91:443",   // DC4
            "91.108.56.130:443",    // DC5
        ];

        for endpoint in &dc_endpoints {
            if let Ok(addr) = endpoint.parse() {
                if TcpStream::connect_timeout(
                    &addr,
                    Duration::from_secs(8),
                ).is_ok() {
                    return true;
                }
            }
        }

        false
    })
    .await;

    match result {
        Ok(available) => (StatusCode::OK, Json(available)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
