watch *ARGS:
  #!/usr/bin/env bash
  rustup override set nightly
  export RUSTFLAGS="${RUSTFLAGS:-} --cfg tokio_unstable"
  systemfd --no-pid -s http::0.0.0.0:8998 --  cargo watch -q -c -w . -x "+nightly run --all-features {{ARGS}}"

test-watch *ARGS:
  #!/usr/bin/env bash
  rustup override set nightly
  export RUSTFLAGS="${RUSTFLAGS:-} --cfg tokio_unstable"
  cargo watch -c -w . -x "+nightly nextest run --all-features --verbose {{ARGS}}"

