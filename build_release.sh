#!/usr/bin/env bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/feh_sim_seed.wasm --no-modules --out-dir ./pkg --out-name package
../binaryen/bin/wasm-opt pkg/package_bg.wasm -o pkg/package_bg.wasm
