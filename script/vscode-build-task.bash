#!/usr/bin/env bash

set -o errexit
set -o xtrace

cargo build
cargo test
cargo doc
