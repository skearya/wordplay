# syntax=docker/dockerfile:1

FROM lukemathwalker/cargo-chef:latest-rust-1.94.1-slim AS chef

WORKDIR /app

FROM chef AS planner

COPY server .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY server .
RUN cargo build --release --bin server

FROM node:24.12.0-alpine AS builder-web

WORKDIR /app/web

COPY web/package.json web/package-lock.json ./
RUN npm ci

COPY web .
COPY server/bindings ../server/bindings

RUN npm run check
RUN npm run build

FROM debian:trixie-slim AS runtime

RUN groupadd -g 1001 appgroup && \
    useradd -u 1001 -g appgroup -m -d /home/appuser -s /bin/bash appuser

COPY --from=builder --chown=appuser:appgroup /app/target/release/server /usr/local/bin/server
COPY --from=builder-web --chown=appuser:appgroup /app/web/build assets

USER appuser
EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/server"]
