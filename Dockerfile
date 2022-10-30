
FROM lukemathwalker/cargo-chef:latest-rust-1.59.0 as chef

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

FROM debian:bullseye-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates ffmpeg \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/cyberpunk cyberpunk

COPY configuration configuration
ENV APP_ENVIRONMENT production

ENTRYPOINT ["./cyberpunk"]
