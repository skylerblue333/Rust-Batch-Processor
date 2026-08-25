# Changelog

## Unreleased

### Added
- Atomic multi-item enqueue with duplicate-ID rejection.
- Explicit payload, batch, queue, and tracked-ID bounds.
- FIFO processed-item responses and processor stats.
- Health and readiness endpoints.
- Precise HTTP error mapping for duplicates, validation, capacity, and unavailable state.
- Rustfmt, Clippy, tests, release build, cargo-audit, Docker build, and non-root CI gates.
- Security and SKYCOIN4444 integration documentation.

### Changed
- Preserved the native Rust batch queue while replacing panic-prone mutex access with explicit errors.
- Migrated the HTTP adapter from the older Actix dependency line to Axum.
- Replaced unbounded processed-item retention with a processed counter.
- Repositioned the repository as an engineering-beta bounded in-memory batch service.

### Known limitations
- No durable storage, distributed execution, leases, retries, dead-letter queue, authentication, TLS termination, HA, or verified production deployment.
