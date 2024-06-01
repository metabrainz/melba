FROM rust as builder

COPY . /app

WORKDIR /app

RUN cargo build --release

FROM gcr.io/distroless/cc-debian11

COPY --from=builder /app/target/release/mb-exurl-ia-service /app/mb-exurl-ia-service
WORKDIR /app

CMD ["./mb-exurl-ia-service"]