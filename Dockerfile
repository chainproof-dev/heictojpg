# Using cargo-chef for faster builds
FROM rust:1.83-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# Install build dependencies
RUN apt-get update && apt-get install -y \
    libheif-dev \
    libjpeg62-turbo-dev \
    clang \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --bin heictojpg

# Runtime image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libheif1 \
    libjpeg62-turbo \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/heictojpg /usr/local/bin/
COPY --from=builder /app/static /app/static

# Create non-root user
RUN useradd -m -u 1000 -U appuser
USER appuser

ENV SERVER_PORT=3000
EXPOSE 3000

CMD ["heictojpg"]
