# TODO: Optimize image by switching to Alpine images instead
FROM rust:1.84.1 AS builder
WORKDIR /src
COPY /backend ./
RUN cargo build --release && cargo test

FROM debian:bookworm-slim as runner
ENV CHAT_APP_ENVIRONMENT=prod
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl \
    ca-certificates \
    && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*
WORKDIR /config
COPY /config/* ./
WORKDIR /app
COPY --from=builder /src/target/release/chat-backend /app/chat-backend
CMD ["./chat-backend"]
