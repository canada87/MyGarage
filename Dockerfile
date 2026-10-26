# syntax=docker/dockerfile:1

# ---- Frontend (SvelteKit static build) ----
FROM node:24-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# ---- Backend (Rust, frontend embedded in the binary) ----
FROM rust:1-bookworm AS backend
WORKDIR /app/backend
# Build dependencies first so they are cached between source changes.
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir -p src ../frontend/build \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src
COPY backend/ ./
COPY --from=frontend /app/frontend/build ../frontend/build
RUN touch src/main.rs && cargo build --release

# ---- Runtime ----
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*
COPY --from=backend /app/backend/target/release/mygarage /usr/local/bin/mygarage
ENV MYGARAGE_DATA_DIR=/data \
    MYGARAGE_DOCS_DIR=/documents \
    MYGARAGE_LISTEN=0.0.0.0:8080 \
    TZ=Europe/Rome
VOLUME ["/data", "/documents"]
EXPOSE 8080
CMD ["mygarage"]
