use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use sky_batch_processor::{BatchItem, BatchProcessor, ProcessorError, ProcessorStats};
use std::{env, sync::Arc};

#[derive(Debug, Serialize)]
struct EnqueueResponse {
    queued: usize,
}

#[derive(Debug, Serialize)]
struct ProcessResponse {
    processed: Vec<BatchItem>,
    stats: ProcessorStats,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Debug, Serialize)]
struct ReadyResponse {
    ready: bool,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
    detail: String,
}

struct ApiError(ProcessorError);

impl From<ProcessorError> for ApiError {
    fn from(value: ProcessorError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match self.0 {
            ProcessorError::DuplicateId(_) => (StatusCode::CONFLICT, "duplicate_id"),
            ProcessorError::CapacityExceeded(_) => (StatusCode::TOO_MANY_REQUESTS, "capacity_exceeded"),
            ProcessorError::InvalidConfig(_) | ProcessorError::InvalidItem(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "invalid_request")
            }
            ProcessorError::StateUnavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "state_unavailable")
            }
        };
        (status, Json(ErrorResponse { error: code, detail: self.0.to_string() })).into_response()
    }
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "sky-batch-processor",
    })
}

async fn ready(State(processor): State<Arc<BatchProcessor>>) -> Result<Json<ReadyResponse>, ApiError> {
    processor.stats()?;
    Ok(Json(ReadyResponse { ready: true }))
}

async fn stats(State(processor): State<Arc<BatchProcessor>>) -> Result<Json<ProcessorStats>, ApiError> {
    Ok(Json(processor.stats()?))
}

async fn enqueue(
    State(processor): State<Arc<BatchProcessor>>,
    Json(items): Json<Vec<BatchItem>>,
) -> Result<(StatusCode, Json<EnqueueResponse>), ApiError> {
    let queued = processor.enqueue_many(items)?;
    Ok((StatusCode::ACCEPTED, Json(EnqueueResponse { queued })))
}

async fn process_batch(
    State(processor): State<Arc<BatchProcessor>>,
) -> Result<Json<ProcessResponse>, ApiError> {
    let processed = processor.process_batch()?;
    let stats = processor.stats()?;
    Ok(Json(ProcessResponse { processed, stats }))
}

fn env_usize(name: &str, default: usize) -> Result<usize, String> {
    match env::var(name) {
        Ok(value) => value
            .parse::<usize>()
            .map_err(|_| format!("{name} must be a positive integer")),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => Err(format!("{name} must be valid UTF-8")),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let batch_size = env_usize("BATCH_SIZE", 100)?;
    let max_queue = env_usize("MAX_QUEUE", 10_000)?;
    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_owned());
    let processor = Arc::new(BatchProcessor::new(batch_size, max_queue)?);

    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v1/stats", get(stats))
        .route("/api/v1/enqueue", post(enqueue))
        .route("/api/v1/process", post(process_batch))
        .with_state(processor);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    println!("sky-batch-processor listening on {bind_addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
