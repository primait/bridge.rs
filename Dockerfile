FROM rust:1.72

WORKDIR /code

COPY entrypoint /code/entrypoint

RUN groupadd -g 1000 app && \
    useradd -g 1000 -u 1000 --system --create-home app && \
    chown -R app:app /code && \
    cargo install cargo-make && \
    rustup component add clippy rustfmt

USER app

ENTRYPOINT ["./entrypoint"]
