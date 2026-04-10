FROM node:22-bookworm AS builder

ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH=/usr/local/cargo/bin:/usr/local/rustup/bin:${PATH}

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl build-essential pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain stable
RUN cargo install wasm-pack --version 0.13.1

WORKDIR /app

COPY Cargo.toml Cargo.lock package.json package-lock.json VERSION README.md ./
COPY common ./common
COPY frontend ./frontend
COPY game-logic ./game-logic
COPY game-logic-wasm ./game-logic-wasm
COPY scripts ./scripts
COPY server ./server
COPY shared ./shared

RUN ./scripts/install-app-deps.sh
RUN ./scripts/build-app.sh

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

ENV MATTER_SERVER_HOST=0.0.0.0
ENV MATTER_SERVER_PORT=3000
ENV MATTER_SERVER_DATABASE_PATH=/data/battlecontrol.db

COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/frontend/build /app/frontend/build

EXPOSE 3000

CMD ["/app/server"]
