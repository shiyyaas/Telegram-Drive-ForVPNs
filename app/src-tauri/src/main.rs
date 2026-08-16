use std::sync::Arc;
use app_lib::{create_router, init_telegram_state, bandwidth::BandwidthManager, server};

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting headless application server...");

    let telegram_state = init_telegram_state();
    let bw_manager = Arc::new(BandwidthManager::new());

    // Start Actix Streaming Server in background thread
    let state_for_actix = Arc::new(telegram_state.clone());
    std::thread::spawn(move || {
        let sys = actix_rt::System::new();
        sys.block_on(async move {
            match server::start_server(state_for_actix, server::STREAMING_PORT).await {
                Ok(server) => {
                    log::info!("Actix streaming server running on port {}", server::STREAMING_PORT);
                    server.await.ok();
                }
                Err(e) => log::error!("Streaming server failed: {}", e),
            }
        });
    });

    let app = create_router(telegram_state, bw_manager);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8550);

    let addr = format!("0.0.0.0:{}", port);
    log::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TcpListener");

    axum::serve(listener, app)
        .await
        .expect("Axum server error");
}
