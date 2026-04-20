# ---- builder ----
FROM rust:1.82-slim AS builder
WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy source and rebuild
COPY src ./src
COPY tests ./tests
RUN touch src/main.rs && cargo build --release

# ---- runtime ----
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fusion-tracker /app/fusion-tracker
COPY static ./static
COPY data ./data

ENV PORT=8080
EXPOSE 8080
CMD ["/app/fusion-tracker"]
