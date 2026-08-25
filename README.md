# Sky Batch Processor

A bounded in-memory Rust batch queue with a small Axum HTTP boundary. The project preserves the repository's original FIFO batch-processing concept while hardening validation, capacity management, duplicate handling, observability, error semantics, CI, and container packaging.

## Implemented behavior

- Native Rust queue core with a single mutex-protected state boundary.
- Configurable processing batch size (`BATCH_SIZE`, default `100`, maximum `1000`).
- Configurable queue capacity (`MAX_QUEUE`, default `10000`, maximum `100000`).
- Atomic multi-item enqueue: either the entire request is accepted or none of it is.
- Caller-supplied `u64` item IDs with duplicate rejection across queued and previously processed items.
- Payloads must contain 1–16,384 UTF-8 bytes.
- FIFO processing up to the configured batch size.
- Bounded tracked-ID set (100,000 maximum) to make the current in-memory idempotency boundary explicit.
- Poisoned mutexes surface as service-unavailable errors rather than panicking through `unwrap()`.
- Health, readiness, queue stats, enqueue, and process endpoints.
- 409 duplicate-ID, 422 validation, 429 capacity, and 503 state-unavailable HTTP mappings.
- Non-root container image.

## API

### `GET /health`
Liveness response.

### `GET /ready`
Checks that processor state can be read.

### `GET /api/v1/stats`
Returns queued count, processed count, tracked IDs, configured batch size, and queue capacity.

### `POST /api/v1/enqueue`
Accepts a JSON array:

```json
[
  {"id": 101, "payload": "first"},
  {"id": 102, "payload": "second"}
]
```

### `POST /api/v1/process`
Dequeues up to `BATCH_SIZE` items in FIFO order and returns the processed items plus updated stats.

## Run locally

```bash
cargo run
```

Configuration example:

```bash
BATCH_SIZE=50 MAX_QUEUE=5000 BIND_ADDR=127.0.0.1:8080 cargo run
```

## Verification

CI requires:

```bash
cargo generate-lockfile
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
cargo build --release --locked
cargo audit
```

CI also builds the Docker image and verifies non-root execution.

## Architecture

`src/lib.rs` contains the reusable processor and its invariants. `src/main.rs` is the Axum adapter. Keeping the queue domain separate from HTTP allows the core to be tested without a server and reused through another adapter later.

The dependency line intentionally uses Axum rather than retaining the older Actix stack so the product can stay on a current HTTP/2 dependency path and be audited rather than suppressing known dependency advisories.

## SKYCOIN4444 integration

The component can provide a bounded local batch-work boundary for analytics, notifications, exports, or other adapters. Production ecosystem integration should use a stable API/client wrapper and should not copy the queue implementation into the flagship codebase.

## Status and limitations

**Status: Engineering Beta.** Code/container verification is being established; deployment is not verified.

This implementation is process-local and in-memory. Restarting loses the queue and tracked IDs. It does not provide durable persistence, distributed workers, leases, retries, dead-letter queues, exactly-once execution, tenant isolation, authentication, TLS termination, HA, or production deployment. The 100,000 tracked-ID cap means long-running deployments must rotate/restart or adopt durable idempotency storage rather than treating the current service as an unlimited queue.

See `SECURITY.md` and `CHANGELOG.md` for product/security boundaries.
