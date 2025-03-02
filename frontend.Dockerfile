FROM node:23-bookworm AS builder
WORKDIR /src
COPY /frontend ./
RUN npm install
RUN npm run build

FROM node:23-bookworm-slim as runner
ENV CHAT_APP_ENVIRONMENT=prod
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl \
    ca-certificates \
    && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*
WORKDIR /config
COPY /config/* ./
WORKDIR /app
COPY --from=builder /src/package*.json ./
COPY --from=builder /src/dist ./dist
COPY --from=builder /src/vite.config.ts ./
RUN npm install
CMD ["npm", "run", "preview", "--", "--host"]
