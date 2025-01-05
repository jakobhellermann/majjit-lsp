build:
  cargo build

lint:
  cargo clippy --workspace --all-targets -- --deny warnings
