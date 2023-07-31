FROM lukemathwalker/cargo-chef:latest-rust-1.71.0 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
ENV RUSTFLAGS "--cfg uuid_unstable"
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
ENV RUSTFLAGS "--cfg uuid_unstable"
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE true
ENV RUSTFLAGS "--cfg uuid_unstable"
RUN cargo build --release --bin newsletter

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/newsletter newsletter
COPY configuration configuration
EXPOSE 8000
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./newsletter"]