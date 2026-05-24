server-fmt:
  cargo fmt --manifest-path pulse-server/Cargo.toml

server-run:
  cargo run --manifest-path pulse-server/Cargo.toml

server-test:
  cargo test --manifest-path pulse-server/Cargo.toml

server-build:
  cargo build --manifest-path pulse-server/Cargo.toml --release

server-run-release:
  ./pulse-server/target/release/pulse-backend