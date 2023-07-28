FROM gcr.io/distroless/cc
WORKDIR /app
COPY ./target/release/photomanager .

EXPOSE 8998

ENV RUST_BACKTRACE=1

CMD ["./photomanager"]
