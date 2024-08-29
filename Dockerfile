FROM lukemathwalker/cargo-chef:latest-rust-1.80.1 AS chef
WORKDIR /app

FROM chef AS planner
COPY server .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY server .
RUN cargo build --release

FROM node:21 as client-builder
WORKDIR /client
ENV NODE_ENV production
COPY client/package.json client/package-lock.json ./
RUN npm ci && npm cache clean --force
COPY client .
RUN npm run build

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends libssl3 && apt-get clean && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/server /app/server
COPY --from=client-builder /client/dist /app/static
EXPOSE 3021
ENTRYPOINT ["/app/server"]
