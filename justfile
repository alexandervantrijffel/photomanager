watch *ARGS:
  rustup override set nightly
  RUST_BACKTRACE=1 systemfd --no-pid -s http::0.0.0.0:8998 --  cargo watch -q -c --ignore '**/generated_at_build.rs' -w . -x "+nightly run --all-features {{ARGS}}"
