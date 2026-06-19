use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use app::BatchProcessor;

#[derive(Deserialize)]
struct EnqueueRequest {
    items: Vec<String>,
}

#[derive(Serialize)]
struct ProcessResponse {
    processed: usize,
    total_processed: usize,
}

async fn enqueue(
    processor: web::Data<Arc<BatchProcessor>>,
    req: web::Json<EnqueueRequest>,
) -> impl Responder {
    for (i, item) in req.items.iter().enumerate() {
        processor.enqueue(app::BatchItem { id: i as u64, payload: item.clone() });
    }
    HttpResponse::Ok().json(serde_json::json!({ "queued": req.items.len() }))
}

async fn process_batch(processor: web::Data<Arc<BatchProcessor>>) -> impl Responder {
    let processed = processor.process_batch();
    HttpResponse::Ok().json(ProcessResponse {
        processed,
        total_processed: processor.processed_count(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let processor = Arc::new(BatchProcessor::new(100));
    println!("Rust-Batch-Processor running on :8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(processor.clone()))
            .route("/api/v1/enqueue", web::post().to(enqueue))
            .route("/api/v1/process", web::post().to(process_batch))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
