# TODO: Optimize image by switching to Alpine images instead
# Rust backend
FROM rust:1.84.1 as backend-builder
WORKDIR /src
COPY /backend ./
RUN cargo build --release && cargo test

# TypeScript frontend
FROM node:23-bookworm as frontend-builder
WORKDIR /src
COPY /frontend ./
RUN npm install
RUN npm run build

# Runner
FROM node:23-bookworm-slim as runner
ENV CHAT_APP_ENVIRONMENT=prod
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl \
    ca-certificates \
    && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*
WORKDIR /config
COPY /config/* ./
WORKDIR /app
COPY --from=backend-builder /src/target/release/chat-backend /app/chat-backend
COPY --from=frontend-builder /src/package*.json ./
COPY --from=frontend-builder /src/dist ./dist
COPY --from=frontend-builder /src/vite.config.ts ./
COPY ./docker-entrypoint.sh ./
RUN npm install && chmod +x docker-entrypoint.sh
CMD ["./docker-entrypoint.sh"]
