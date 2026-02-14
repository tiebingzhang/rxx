use crate::db::Database;
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub id: String,
    pub ipv6: String,
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub id: String,
    pub ipv6: String,
    pub peer_id: String,
}

#[derive(Serialize)]
pub struct UpdateResponse {
    pub peer_ipv6: String,
}

struct AppState {
    db: Arc<Mutex<Database>>,
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Response {
    let db = state.db.lock().unwrap();
    match db.register(&req.id, &req.ipv6) {
        Ok(true) => {
            println!("Registered: {} -> {}", req.id, req.ipv6);
            StatusCode::OK.into_response()
        }
        Ok(false) => {
            println!("Duplicate ID: {}", req.id);
            StatusCode::CONFLICT.into_response()
        }
        Err(e) => {
            eprintln!("Register error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn update(State(state): State<Arc<AppState>>, Json(req): Json<UpdateRequest>) -> Response {
    let db = state.db.lock().unwrap();

    if let Err(e) = db.update(&req.id, &req.ipv6) {
        eprintln!("Update error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    match db.get_ipv6(&req.peer_id) {
        Ok(Some(peer_ipv6)) => {
            println!(
                "Updated {} -> {}, resolved {} -> {}",
                req.id, req.ipv6, req.peer_id, peer_ipv6
            );
            Json(UpdateResponse { peer_ipv6 }).into_response()
        }
        Ok(None) => {
            println!("Peer not found: {}", req.peer_id);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(e) => {
            eprintln!("Get peer error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn run_server(db_path: &str, port: u16) -> Result<()> {
    let db = Database::open(db_path)?;
    let state = Arc::new(AppState {
        db: Arc::new(Mutex::new(db)),
    });

    let app = Router::new()
        .route("/register", post(register))
        .route("/update", post(update))
        .with_state(state);

    let addr = format!("[::]:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Server listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
