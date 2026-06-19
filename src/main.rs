use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
struct ServiceState {
    domain: String,
    processed: usize,
}

struct AppState {
    state: Mutex<ServiceState>,
}

async fn health(data: web::Data<AppState>) -> impl Responder {
    let state = data.state.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({"status": "ok", "domain": state.domain, "processed": state.processed}))
}

async fn process(data: web::Data<AppState>) -> impl Responder {
    let mut state = data.state.lock().unwrap();
    state.processed += 1;
    HttpResponse::Accepted().json(serde_json::json!({"status": "processing"}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        state: Mutex::new(ServiceState {
            domain: "processor".to_string(),
            processed: 0,
        }),
    });

    println!("Starting server on :8080");
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/health", web::get().to(health))
            .route("/process", web::post().to(process))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
