FROM rust:1.54 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src/ ./src/

RUN cargo build --release

RUN strip /app/target/release/dinkelberg

FROM debian:buster-slim

WORKDIR /app

COPY --from=builder /app/target/release/dinkelberg /app

CMD [ "/app/dinkelberg" ]