server-fmt:
  cargo fmt --manifest-path pulse-server/Cargo.toml

server-build:
  cargo build --manifest-path pulse-server/Cargo.toml --release

development-server-run:
  set -a && . ./.development.env && set +a && cargo run --manifest-path pulse-server/Cargo.toml

development-server-test:
  set -a && . ./.development.env && set +a && cargo test --manifest-path pulse-server/Cargo.toml

development-server-run-release:
  set -a && . ./.development.env && set +a && ./pulse-server/target/release/pulse-backend
