
FROM lukemathwalker/cargo-chef:latest-rust-1.83.0 AS chef

WORKDIR /app

RUN apt update && apt install lld clang -y

# -----------------------------

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

# -----------------------------

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . . 

RUN cargo build --release --bin cyberpunk

# ----------------------------

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates ffmpeg curl \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/cyberpunk cyberpunk

COPY config config
ENV APP_ENVIRONMENT=production

# Cloud Run expects the app to listen on 0.0.0.0:$PORT
ENV APP_APPLICATION__HOST=0.0.0.0

# Health check for Cloud Run
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT:-8080}/health || exit 1

ENTRYPOINT ["./cyberpunk"]
