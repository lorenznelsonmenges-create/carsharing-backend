use axum::{
    extract::State,
    routing::{get, post}, // 'post' hinzugefügt
    Json, Router,
    http::Method,
};
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

pub mod carsharing;
use crate::carsharing::CarSharing;

type AppState = Arc<Mutex<CarSharing>>;

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(CarSharing::new()));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST]) // 'POST' hinzugefügt
        .allow_headers(vec![axum::http::header::CONTENT_TYPE]) // Wichtig für POST
        .allow_origin(Any);

    let app = Router::new()
        .route("/api/state", get(get_state).post(update_state)) // .post(update_state) hinzugefügt
        .with_state(shared_state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Backend lauscht auf http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

/// Gibt den aktuellen CarSharing-Zustand als JSON zurück
async fn get_state(State(state): State<AppState>) -> Json<CarSharing> {
    let car_sharing_data = state.lock().unwrap().clone();
    Json(car_sharing_data)
}

/// Empfängt einen neuen Zustand und überschreibt den alten
async fn update_state(
    State(state): State<AppState>,
    Json(new_car_sharing_state): Json<CarSharing>,
) {
    let mut car_sharing_data = state.lock().unwrap();
    *car_sharing_data = new_car_sharing_state;
    println!("Neuen State vom Frontend empfangen und gespeichert.");
}
