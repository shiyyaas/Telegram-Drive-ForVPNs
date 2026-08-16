# Stage 1: Frontend Builder
FROM node:20-alpine AS frontend-builder
WORKDIR /app
COPY app/package*.json ./
RUN npm ci
COPY app/ ./
RUN npm run build

# Stage 2: Backend Builder
FROM rust:1.75-slim AS backend-builder
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY app/src-tauri ./src-tauri
WORKDIR /usr/src/app/src-tauri
RUN cargo build --release

# Stage 3: Runtime Bundle
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /usr/src/app/src-tauri/target/release/app /app/server
COPY --from=frontend-builder /app/dist /app/dist

ENV PORT=8550
ENV STATIC_DIR=/app/dist
ENV DATA_DIR=/app/data

EXPOSE 8550

CMD ["/app/server"]
