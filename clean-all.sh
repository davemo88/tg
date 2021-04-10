#!/bin/bash

cargo clean --manifest-path tglib/Cargo.toml
cargo clean --manifest-path rbtr-public/Cargo.toml
cargo clean --manifest-path rbtr-private/Cargo.toml
cargo clean --manifest-path player-wallet/Cargo.toml
cargo clean --manifest-path player-cli/Cargo.toml
cargo clean --manifest-path nmc-id/Cargo.toml
cargo clean --manifest-path referee-signer/Cargo.toml
