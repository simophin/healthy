FROM rust:bookworm

WORKDIR /usr/src/healthy
COPY . .

RUN cargo build --release

FROM debian:bookworm
COPY --from=0 /usr/src/healthy/target/release/healthy /usr/local/bin/healthy

ENV RUST_LOG=info
ENV LISTEN_ADDR=0.0.0.0:3400

ENTRYPOINT ["/usr/local/bin/healthy"]
