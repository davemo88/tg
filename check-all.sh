#!/bin/bash

cargo check --manifest-path tglib/Cargo.toml
cargo check --manifest-path rbtr-public/Cargo.toml
cargo check --manifest-path rbtr-private/Cargo.toml
cargo check --manifest-path player-wallet/Cargo.toml
cargo check --manifest-path player-cli/Cargo.toml
cargo check --manifest-path nmc-id/Cargo.toml
cargo check --manifest-path referee-signer/Cargo.toml
