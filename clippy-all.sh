#!/bin/bash

cargo clippy --manifest-path tglib/Cargo.toml
cargo clippy --manifest-path rbtr-public/Cargo.toml
cargo clippy --manifest-path rbtr-private/Cargo.toml
cargo clippy --manifest-path player-wallet/Cargo.toml
cargo clippy --manifest-path player-cli/Cargo.toml
cargo clippy --manifest-path nmc-id/Cargo.toml
cargo clippy --manifest-path referee-signer/Cargo.toml
