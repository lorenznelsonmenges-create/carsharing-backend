         use axum::{
             extract::State,
             routing::get,
             Json, Router,
             http::Method,
         };
         use std::sync::{Arc, Mutex};
         use tower_http::cors::{Any, CorsLayer};
   
        // Lade das Carsharing-Modul, das wir gerade kopiert haben
        pub mod carsharing;
        use crate::carsharing::CarSharing;
   
        // Der AppState, der in allen Handlern geteilt wird
        type AppState = Arc<Mutex<CarSharing>>;
   
        #[tokio::main]
        async fn main() {
            // Initialisiere den CarSharing-Zustand
            let shared_state = Arc::new(Mutex::new(CarSharing::new()));
   
            // CORS-Layer hinzufügen, damit dein Frontend vom Backend laden darf
            let cors = CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any);
   
            // Die Routen für unsere API definieren
            let app = Router::new()
                .route("/api/state", get(get_state))
                // .route("/api/state", post(update_state)) // POST fügen wir später hinzu
                .with_state(shared_state)
                .layer(cors);
   
            // Den Server starten
            let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
            println!("Backend lauscht auf http://127.0.0.1:3000");
            axum::serve(listener, app).await.unwrap();
        }
   
        /// Gibt den aktuellen CarSharing-Zustand als JSON zurück
        async fn get_state(State(state): State<AppState>) -> Json<CarSharing> {
            let car_sharing_data = state.lock().unwrap().clone();
            Json(car_sharing_data)
        }

