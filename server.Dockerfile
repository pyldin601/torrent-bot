FROM rust:1.80.0

WORKDIR /code

COPY Cargo.lock /code/Cargo.lock
COPY Cargo.toml /code/Cargo.toml

COPY crates /code/crates

RUN \
    cargo build --release --bin torrent-bot-server && \
    mv target/*/torrent-bot-server torrent-bot-server && \
    rm -rf target

FROM rust:1.80.0

COPY --from=0 /code/torrent-bot-server /torrent-bot-server

CMD ["/torrent-bot-server"]
