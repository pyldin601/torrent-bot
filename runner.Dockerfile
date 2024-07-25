FROM rust:1.80.0

WORKDIR /code

COPY Cargo.lock /code/Cargo.lock
COPY Cargo.toml /code/Cargo.toml

COPY crates /code/crates

RUN \
    cargo build --release --bin torrent-bot-runner && \
    mv target/*/torrent-bot-runner torrent-bot-runner && \
    rm -rf target

FROM rust:1.80.0

COPY --from=0 /code/torrent-bot-runner /torrent-bot-runner

CMD ["/torrent-bot-runner"]
