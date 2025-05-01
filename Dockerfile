FROM rust:latest AS build-env

WORKDIR /app
COPY . /app

RUN rustc --version

# Copy only Cargo files to cache deps
COPY Cargo.toml Cargo.lock ./

# Fetch, build and cache dependencies
RUN cargo fetch
RUN cargo build --release --locked || true

COPY . .
RUN cargo build --release --locked

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/photomanager /

EXPOSE 8998

ENV RUST_BACKTRACE=1

USER 1000:1000

CMD ["./photomanager"]
