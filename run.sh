#!/usr/bin/env bash

set -e

pushd $(git rev-parse --show-toplevel)
  cd frontend
  cargo build --target wasm32-unknown-unknown --release
  wasm-bindgen ../target/wasm32-unknown-unknown/release/frontend.wasm --out-dir out --target web

  cd ../server
  cargo run
popd
