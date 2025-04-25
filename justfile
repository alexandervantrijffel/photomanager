watch *ARGS:
  #!/usr/bin/env bash
  rustup override set nightly
  export RUSTFLAGS="${RUSTFLAGS:-} --cfg tokio_unstable"
  systemfd --no-pid -s http::0.0.0.0:8998 --  cargo watch -q -c --ignore '**/generated_at_build.rs' -w . -x "+nightly run --all-features {{ARGS}}"
