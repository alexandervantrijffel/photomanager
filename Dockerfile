FROM rust:latest AS build-env

WORKDIR /app
COPY . /app

RUN cargo version
RUN rustc --version

RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/photomanager /

EXPOSE 8998

ENV RUST_BACKTRACE=1

USER 1000:1000

CMD ["./photomanager"]
