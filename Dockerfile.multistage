FROM rust:1.71.0 as build-env
WORKDIR /app
COPY . /app

RUN cargo build --release
# RUN cargo clippy --verbose -- -D warnings
# RUN cargo test --verbose
# RUN cargo install cargo-audit
# RUN cargo audit

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/photomanager /

EXPOSE 8998

ENV RUST_BACKTRACE=1

CMD ["./photomanager"]
