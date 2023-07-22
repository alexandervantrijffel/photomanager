FROM rust:1.71.0 as build-env
WORKDIR /app
COPY . /app

RUN cargo install cargo-audit
RUN cargo build --release --no-default-features
RUN cargo clippy --verbose -- -D warnings
RUN cargo audit

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/photomanager /

EXPOSE 8998

ENV RUST_BACKTRACE=1

CMD ["./photomanager"]
