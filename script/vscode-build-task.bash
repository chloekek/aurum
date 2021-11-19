#!/usr/bin/env bash

set -o errexit
set -o xtrace

cargo build
cargo build --release
cargo test
cargo test --release
cargo doc
