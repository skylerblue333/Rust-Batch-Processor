FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo generate-lockfile && cargo build --release --locked

FROM debian:bookworm-slim
WORKDIR /app
RUN useradd --system --uid 10001 --create-home app
COPY --from=builder /app/target/release/sky-batch-processor /usr/local/bin/sky-batch-processor
USER app
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/sky-batch-processor"]
