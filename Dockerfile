FROM rust:1.68.0 as build-env
WORKDIR /app
COPY . /app
RUN cargo build --release --no-default-features

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/photomanager /
CMD ["./photomanager"]
