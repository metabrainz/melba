FROM rust:latest

WORKDIR /usr/src/mb-exurl-ia-service

COPY . .

RUN cargo install --path .

CMD ["mb-exurl-ia-service"]

