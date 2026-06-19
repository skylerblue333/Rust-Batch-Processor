FROM rust:1.73-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/app ./app
EXPOSE 8080
CMD ["./app"]
