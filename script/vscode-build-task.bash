#!/usr/bin/env bash

set -o errexit
set -o xtrace

cargo build --profile dev
cargo test --profile dev

cargo build --profile release
cargo test --profile release

cargo doc
