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
    git \
    build-essential \
    cmake \
    pkg-config \
    clang \
    llvm \
    nasm \
    libjpeg62-turbo-dev \
    libde265-dev \
    libx265-dev \
    libaom-dev \
    && rm -rf /var/lib/apt/lists/*

# Build and install libheif 1.18.2 from source (matching CI)
RUN git clone --depth 1 --branch v1.18.2 https://github.com/strukturag/libheif.git && \
    cd libheif && \
    mkdir build && cd build && \
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX=/usr/local \
        -DWITH_EXAMPLES=OFF \
        -DWITH_GDK_PIXBUF=OFF \
        -DWITH_RAV1E=OFF \
        -DWITH_DAV1D=OFF && \
    make -j$(nproc) && \
    make install && \
    ldconfig && \
    cd ../.. && rm -rf libheif

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --bin heictojpg

# Runtime image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies (libraries linked against)
RUN apt-get update && apt-get install -y \
    libjpeg62-turbo \
    libde265-0 \
    libx265-199 \
    libaom3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled libheif libraries from builder
COPY --from=builder /usr/local/lib/libheif.so* /usr/local/lib/
# Update library cache
RUN ldconfig

COPY --from=builder /app/target/release/heictojpg /usr/local/bin/
COPY --from=builder /app/static /app/static

# Create non-root user
RUN useradd -m -u 1000 -U appuser
USER appuser

ENV SERVER_PORT=3000
EXPOSE 3000

CMD ["heictojpg"]
