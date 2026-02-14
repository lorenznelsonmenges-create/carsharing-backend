use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
    http::Method,
};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions}; // NEU
use tower_http::cors::{Any, CorsLayer};

pub mod carsharing;
use crate::carsharing::CarSharing;

// Der AppState ist jetzt der Datenbank-Pool
type AppState = SqlitePool;

#[tokio::main]
async fn main() {
    // --- NEU: Datenbankverbindung aufbauen ---
    let db_url = "sqlite:data/carsharing.db";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Kann Datenbank nicht verbinden");

    // --- NEU: Tabelle erstellen, falls sie nicht existiert ---
    sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS carsharing_state (
                id INTEGER PRIMARY KEY NOT NULL,
                state_json TEXT NOT NULL
            );
            "#,
    )
    .execute(&pool)
    .await
    .expect("Tabelle konnte nicht erstellt werden");

    // --- NEU: Sicherstellen, dass eine Zeile zum Speichern existiert ---
    let initial_state_json = serde_json::to_string(&CarSharing::new()).unwrap();
    sqlx::query(
        "INSERT OR IGNORE INTO carsharing_state (id, state_json) VALUES (1, ?);"
    )
    .bind(initial_state_json)
    .execute(&pool)
    .await
    .expect("Initialer State konnte nicht eingefügt werden");


    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(vec![axum::http::header::CONTENT_TYPE])
        .allow_origin(Any);

    let app = Router::new()
        .route("/api/state", get(get_state).post(update_state))
        .with_state(pool) // Der State ist jetzt der 'pool'
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Backend lauscht auf http://127.0.0.1:3000 und nutzt SQLite");
    axum::serve(listener, app).await.unwrap();
}

/// Liest den Zustand aus der Datenbank
async fn get_state(State(pool): State<AppState>) -> Json<CarSharing> {
    let result: (String,) = sqlx::query_as("SELECT state_json FROM carsharing_state WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();

    let state_json = result.0;
    let car_sharing_data: CarSharing = serde_json::from_str(&state_json).unwrap();

    Json(car_sharing_data)
}

/// Schreibt den neuen Zustand in die Datenbank
async fn update_state(
    State(pool): State<AppState>,
    Json(new_car_sharing_state): Json<CarSharing>,
) {
    let state_json = serde_json::to_string(&new_car_sharing_state).unwrap();

    sqlx::query("UPDATE carsharing_state SET state_json = ? WHERE id = 1")
        .bind(state_json)
        .execute(&pool)
        .await
        .unwrap();

    println!("Neuen State vom Frontend empfangen und in die DB geschrieben.");
}
