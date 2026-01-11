use axum::{
    Json, Router,
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tower_http::cors::{Any, CorsLayer};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Clone, Serialize, Deserialize)]
struct Message {
    user: String,
    text: String,
}

#[derive(Clone)]
struct AppState {
    tx: Sender<Message>,
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let (tx, _) = broadcast::channel(100);
    let state = Arc::new(AppState { tx });

    let cors = CorsLayer::new()
        .allow_origin(Any) // ← для разработки (разрешает все домены)
        .allow_methods(Any) // GET, POST, OPTIONS и т.д.
        .allow_headers(Any) // Content-Type, Authorization и т.д.
        .allow_credentials(false);

    let app = Router::new()
        .route("/send", post(send_message))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server running in http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(msg): Json<Message>,
) -> impl IntoResponse {
    info!("Received message: {}", msg.text);
    state.tx.send(msg).ok();
    StatusCode::OK
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: axum::extract::ws::WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        let json = serde_json::to_string(&msg).unwrap();
        if socket
            .send(axum::extract::ws::Message::Text(json.into()))
            .await
            .is_err()
        {
            break;
        }
    }
}
