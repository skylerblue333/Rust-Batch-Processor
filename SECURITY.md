# Security Policy

Sky Batch Processor is an engineering-beta in-memory queue component.

## Current boundary

The processor validates item count, payload size, duplicate IDs, queue capacity, configuration bounds, and internal-state availability. The supplied container runs as a non-root user and CI audits Rust dependencies.

The HTTP service currently has no authentication or authorization layer and does not terminate TLS. Do not expose it directly to untrusted networks. Place it behind an authenticated gateway or private service boundary when used outside local development.

## Data and availability considerations

- Queue contents and idempotency state are process-local and disappear on restart.
- Processed payloads are returned by the process endpoint; callers should avoid putting secrets or sensitive personal data into payloads unless the surrounding system has an appropriate data policy.
- Capacity errors are deliberate backpressure signals, not evidence of durable buffering.
- A successful process response means items were removed from this in-memory queue, not that downstream side effects completed.

## Unsupported claims

The repository does not provide durable delivery, distributed locking, worker leases, retry/dead-letter semantics, tenant isolation, TLS termination, HA, exactly-once execution, or verified production deployment.

## Reporting

Use GitHub private vulnerability reporting when enabled. Do not place sensitive payloads or credentials in public issues.
