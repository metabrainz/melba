FROM rust:latest as builder

WORKDIR /app

COPY . .

ARG PGHOST
ENV PGHOST=${PGHOST}

ENV RUSTFLAGS='-C target-feature=+crt-static'
ENV SQLX_OFFLINE=true

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --target x86_64-unknown-linux-gnu && \
    cp ./target/x86_64-unknown-linux-gnu/release/mb-ia /mb-ia
FROM scratch

WORKDIR /app
COPY --from=builder /mb-ia ./app

COPY --from=builder /app/config /app/config
CMD ["/app/app"]

